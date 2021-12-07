use async_std::fs::File;
use async_std::io::{copy, BufReader, BufWriter};
use libflatterer::{flatten, flatten_from_jl, FlatFiles, Selector};
use std::fs::File as StdFile;
use std::io::{copy as std_copy, BufReader as StdBufReader};
use surf::http::{Method, Url};
use tempfile::TempDir;
use tide::prelude::*;
use tide::{http, log, utils, Body, Request, Response, StatusCode};
//use async_std::task;
use std::path::PathBuf;
use walkdir::WalkDir;
use uuid::Uuid;

#[derive(Deserialize, Debug, Clone)]
struct Query {
    id: Option<String>,
    file_url: Option<String>,
    array_key: Option<String>,
    json_lines: Option<bool>,
    xlsx: Option<bool>,
    csv: Option<bool>,
    main_table_name: Option<String>,
    inline_one_to_one: Option<bool>,
    json_schema: Option<String>,
    table_schema: Option<String>,
    path_seperator: Option<String>,
    schema_titles: Option<String>,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    env_logger::init();
    let mut app = tide::new();

    app.with(utils::After(|res: Response| async move {
        if let Some(err) = res.error() {
            if res.status() == http::StatusCode::InternalServerError {
                log::error!("Internal Error: {:?}", err)
            } else {
                log::error!("HTTP Error: {:?}", err)
            }
        }
        Ok(res)
    }));

    app.at("/api/zip").get(api);
    app.at("/api/zip").post(api);
    app.at("/api/download_file").post(download_file);
    app.listen("127.0.0.1:8080").await?;

    Ok(())
}

async fn download_file(mut req: Request<()>) -> tide::Result<Response> {
    let mut input: serde_json::Value = req.body_json().await?;

    let url_string = if let Some(url) = input.get_mut("url") {
        if let Some(url_string) = url.as_str() {
            url_string.to_owned()
        } else {"".to_string()}
    } else {"".to_string()};

    let download_value = download(url_string.clone()).await?;

    if download_value.get("error").is_some() {
        let mut res = Response::new(StatusCode::BadRequest);
        let body = Body::from_json(&download_value)?;
        res.set_body(body);
        return Ok(res)
    }

    let mut res = Response::new(StatusCode::Ok);
    let body = Body::from_json(&download_value)?;
    res.set_body(body);

    Ok(res)
}

async fn download(url_string: String) -> tide::Result<serde_json::Value> {
    let uuid = Uuid::new_v4().to_hyphenated();
    let tmp_dir = format!("/tmp/flatterer-{}", uuid);
    async_std::fs::create_dir(&tmp_dir).await?;

    if !url_string.starts_with("http") {
        return Ok(serde_json::json!({"error": "`url` is empty or does not start with `http`"}))
    }

    let url = Url::parse(&url_string)?;
    let req = surf::Request::new(Method::Get, url);
    let client = surf::client();

    let file_response = client.send(req).await?;

    if !file_response.status().is_success() {
        return Ok(serde_json::json!({"error": "file download failed due to bad request status code`", "status_code": file_response.status().to_string()}))
    }

    let download_file = format!("{}/download.json", tmp_dir);
    let file = File::create(&download_file).await?;
    let writer = BufWriter::new(file);

    copy(file_response, writer).await?;

    Ok(serde_json::json!({"id": uuid.to_string ()}))
}

async fn api(req: Request<()>) -> tide::Result<Response> {
    let query: Query = req.query()?;
    let tmp_dir = TempDir::new()?;
    let tmp_dir_path = tmp_dir.path();
    let output_path = tmp_dir_path.join("output");

    let mut download_file: String = "".to_string();

    if let Some(file_url) = &query.file_url {
        let download_value = download(file_url.clone()).await?;
        if let Some(id) = download_value.get("id") {
            if let Some(id_string) = id.as_str() {
                download_file = format!("/tmp/flatterer-{}/download.json", id_string);
            }
        }
        else {
            let mut res = Response::new(StatusCode::BadRequest);
            let body = Body::from_json(&download_value)?;
            res.set_body(body);
            return Ok(res)
        }
    } else if let Some(id) = &query.id {
        download_file = format!("/tmp/flatterer-{}/download.json", id);
        if !std::path::Path::new(&download_file).exists() {
            let mut res = Response::new(StatusCode::BadRequest);
            let body = Body::from_json(&serde_json::json!({"error": "id does not exist, you may need to ask you file to be downloaded again or to upload the file again."}))?;
            res.set_body(body);
            return Ok(res)
        }
    } else {
        let mut res = Response::new(StatusCode::BadRequest);
        let body = Body::from_json(&serde_json::json!({"error": "need to supply either an id or a file_url"}))?;
        res.set_body(body);
        return Ok(res)
    }

    let output_path_to_move = output_path.clone();
    let query_to_move = query.clone();
    let download_file_to_move = download_file.clone();

    //task::spawn_blocking(move || {
    run_flatterer(query_to_move, download_file_to_move, output_path_to_move)?;
    //}).await?;

    let output_path_to_move = output_path.clone();
    let tmp_dir_path_to_move = tmp_dir_path.to_path_buf();

    //task::spawn_blocking(move || {
    zip_output(output_path_to_move, tmp_dir_path_to_move.to_path_buf())?;
    //}).await?;

    let zip_file = tmp_dir_path.join("export.zip");
    let mut res = Response::new(StatusCode::Ok);
    let output = File::open(zip_file).await?;

    let body = Body::from_reader(BufReader::new(output), None); // set the body length

    res.set_body(body);
    res.set_content_type("application/zip");
    res.append_header(
        "Content-Disposition",
        format!("attachment; filename=\"{}.zip\"", "flatterer-download"),
    );

    Ok(res)
}

fn run_flatterer(query: Query, download_file: String, output_path: PathBuf) -> tide::Result<()> {
    let file = StdFile::open(download_file)?;
    let reader = StdBufReader::new(file);
    let flat_files = FlatFiles::new(
        output_path.to_string_lossy().to_string(),
        query.csv.unwrap_or(true),
        query.xlsx.unwrap_or(false),
        true, // force
        query.main_table_name.unwrap_or_else(|| "main".to_string()),
        vec![], // list of json paths to omit object as if it was array
        query.inline_one_to_one.unwrap_or(false),
        query.json_schema.unwrap_or_else(|| "".to_string()),
        query.table_schema.unwrap_or_else(|| "".to_string()),
        query.path_seperator.unwrap_or_else(|| "_".to_string()),
        query.schema_titles.unwrap_or_else(|| "_".to_string()),
    )
    .unwrap();

    if query.json_lines.unwrap_or(false) {
        flatten_from_jl(
            reader,     // reader
            flat_files, // FlatFile instance.
        )?;
    } else {
        let mut selectors = vec![];
        if let Some(array_key) = query.array_key {
            selectors.push(Selector::Identifier(format!("\"{}\"", array_key)));
        };

        flatten(
            reader,     // reader
            flat_files, // FlatFile instance.
            selectors,
        )?;
    }
    Ok(())
}

fn zip_output(output_path: PathBuf, tmp_dir_path: PathBuf) -> tide::Result<()> {
    let zip_file = tmp_dir_path.join("export.zip");

    let file = StdFile::create(&zip_file)?;
    let mut zip = zip::ZipWriter::new(file);

    let options = zip::write::FileOptions::default();

    for entry in WalkDir::new(output_path.clone())
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path == output_path {
            continue;
        }

        if path.is_dir() {
            zip.add_directory(
                path.strip_prefix(output_path.clone())?.to_string_lossy(),
                options,
            )?;
        } else {
            zip.start_file(
                path.strip_prefix(output_path.clone())?.to_string_lossy(),
                options,
            )?;
            let mut file = StdFile::open(path)?;
            std_copy(&mut file, &mut zip)?;
        }
    }
    Ok(())
}
