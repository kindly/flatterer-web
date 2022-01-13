mod buffered_byte_stream;
mod limited_copy;
use async_std::fs::File;
use async_std::io::prelude::*;
use async_std::io::{BufReader, BufWriter};
use limited_copy::copy as limited_copy;
use buffered_byte_stream::BufferedBytesStream;
use libflatterer::{flatten, flatten_from_jl, FlatFiles, Selector};
use std::collections::HashMap;
use std::fs::File as StdFile;
use std::io::{copy as std_copy, BufReader as StdBufReader};
use surf::http::{Method, Url};
use tempfile::TempDir;
use tide::{http, log, utils, Body, Request, Response, StatusCode};
//use async_std::task;
use csv::Reader;
use multer::{Constraints, Multipart, SizeLimit};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env::var;
use std::path::PathBuf;
use uuid::Uuid;
use walkdir::WalkDir;

#[derive(Deserialize, Debug, Clone)]
struct Query {
    id: Option<String>,
    output_format: Option<String>,
    file_url: Option<String>,
    array_key: Option<String>,
    json_lines: Option<bool>,
    xlsx: Option<bool>,
    csv: Option<bool>,
    main_table_name: Option<String>,
    inline_one_to_one: Option<bool>,
    json_schema: Option<String>,
    table_prefix: Option<String>,
    path_seperator: Option<String>,
    schema_titles: Option<String>,
}

fn get_app() -> tide::Result<tide::Server<()>> {
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

    app.at("/api/convert").get(convert);
    app.at("/api/convert").post(convert);
    app.at("/api/convert").put(convert);
    app.at("/about").serve_file("dist/index.html")?;
    app.at("/").serve_file("dist/index.html")?;
    app.at("/").serve_dir("dist/")?;

    Ok(app)
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    env_logger::init();
    clean_tmp()?;

    let app = get_app()?;

    let port = if let Ok(port) = var("PORT") {
        port
    } else {
        "8080".to_string()
    };
    let host = if let Ok(host) = var("HOST") {
        host
    } else {
        "127.0.0.1".to_string()
    };

    app.listen(format!("http://{}:{}", host, port)).await?;

    Ok(())
}


#[derive(Debug, Deserialize, Serialize)]
struct FieldsRecord {
    table_name: String,
    field_name: String,
    field_type: String,
    field_title: Option<String>,
}


async fn download(url_string: String, tmp_dir: &str) -> tide::Result<()> {

    if !url_string.starts_with("http") {
        return Err(tide::Error::from_str(tide::StatusCode::BadRequest, "`url` is empty or does not start with `http`"))
    }

    let url = Url::parse(&url_string)?;
    let req = surf::Request::new(Method::Get, url);
    let client = surf::client();

    let mut file_response = client.send(req).await?;

    if !file_response.status().is_success() {
        return Err(tide::Error::from_str(tide::StatusCode::BadRequest, "file download failed due to bad request status code`"))
    }

    let download_file = format!("{}/download.json", tmp_dir);
    let file = File::create(&download_file).await?;
    let mut writer = BufWriter::new(file);

    limited_copy(&mut file_response, &mut writer).await?;

    Ok(())
}

async fn multipart_upload(req: Request<()>, multipart_boundry: String, tmp_dir: &str) -> tide::Result<Vec<String>> {

    let body_stream = BufferedBytesStream { inner: req };

    let constraints = Constraints::new()
    .size_limit(
        SizeLimit::new()
            .whole_stream(500 * 1024 * 1024)
    );
    let mut multipart = Multipart::with_constraints(body_stream, multipart_boundry.clone(), constraints);

    let mut output = vec![];

    while let Some(mut field) = multipart.next_field().await? {
        let download_file;
        let mut download_output;

        if field.name() == Some("file") {
            download_file = format!("{}/download.json", tmp_dir);
            output.push("file".to_string());
        }
        else if field.name() == Some("fields") {
            download_file = format!("{}/fields.csv", tmp_dir);
            output.push("fields".to_string());
        }
        else if field.name() == Some("tables") {
            download_file = format!("{}/tables.csv", tmp_dir);
            output.push("tables".to_string());
        } else {
            break
        }
        download_output = File::create(&download_file).await?;
        while let Some(chunk) = field.chunk().await? {
            download_output.write_all(&chunk).await?;
        }
    }

    Ok(output)
}

async fn json_request(mut req: Request<()>, tmp_dir: &str) -> tide::Result<()> {
    let download_file = format!("{}/download.json", tmp_dir);
    let mut output = File::create(&download_file).await?;
    limited_copy(&mut req, &mut output).await?;
    Ok(())
}

fn clean_tmp() -> tide::Result<()> {
    for entry in WalkDir::new("/tmp/")
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry
            .file_name()
            .to_string_lossy()
            .starts_with("flatterer-")
        {
            continue;
        }
        if entry.metadata()?.modified()?.elapsed()?.as_secs() > 3600 {
            std::fs::remove_dir_all(&entry.into_path())?;
        }
    }
    Ok(())
}

async fn convert(req: Request<()>) -> tide::Result<Response> {
    clean_tmp()?;
    let query: Query = req.query()?;
    let tmp_dir = TempDir::new()?;
    let tmp_dir_path = tmp_dir.path();
    let output_path = tmp_dir_path.join("output");

    let mut multipart_boundry = "".to_string();
    let mut content_type = "".to_string();

    if let Some(mime) = req.content_type() {
        content_type = mime.essence().to_string();
        if content_type == "multipart/form-data" {
            if let Some(boundry) = mime.param("boundary") {
                multipart_boundry = boundry.to_string()
            }
        }
    }

    let mut json_output;

    if let Some(id) = &query.id {
        json_output = json!({ "id": id })
    } else {  
        let uuid = Uuid::new_v4().to_hyphenated();
        let tmp_dir = format!("/tmp/flatterer-{}", uuid);
        json_output = json!({ "id": uuid.to_string() });
        async_std::fs::create_dir(&tmp_dir).await?;

        let mut uploaded_files = vec![];

        if !multipart_boundry.is_empty() {
            match multipart_upload(req, multipart_boundry, &tmp_dir).await {
                 Err(error) => {json_output = json!({"error": error.to_string()})}
                 Ok(val) => {uploaded_files = val}
            }
        } else if content_type == "application/json" {
            if let Err(error) = json_request(req, &tmp_dir).await {
                json_output = json!({"error": error.to_string()})
            }
            uploaded_files.push("file".to_string());
        } 

        if let Some(file_url) = &query.file_url {
            if let Err(error) = download(file_url.clone(), &tmp_dir).await {
                json_output = json!({"error": error.to_string()})
            }
            uploaded_files.push("file".to_string());
        }

        if !uploaded_files.contains(&"file".to_string()) {
            json_output = json!({"error": "need to supply either an id or filename or supply data in request body"});
        }
    }

    let mut download_file = "".to_string();
    let mut id = "".to_string();

    if let Some(id_value) = json_output.get("id") {
        if let Some(id_string) = id_value.as_str() {
            id = id_string.to_string();
            download_file = format!("/tmp/flatterer-{}/download.json", id_string);
            if !std::path::Path::new(&download_file).exists() {
                json_output = json!({"error": "id does not exist, you may need to ask you file to be downloaded again or to upload the file again."})
            }
        }
    }

    if json_output.get("error").is_some() {
        let mut res = Response::new(StatusCode::BadRequest);
        let body = Body::from_json(&json_output)?;
        res.set_body(body);
        return Ok(res);
    }

    let output_path_copy = output_path.clone();
    let query_copy = query.clone();
    let download_file_copy = download_file.clone();

    let flatterer_result = async_std::task::spawn_blocking(|| -> tide::Result<()> {
        run_flatterer(query_copy, download_file_copy, output_path_copy)?;
        Ok(())
    })
    .await;

    let mut file = File::open(download_file).await?;
    let mut buf = vec![0;1024];
    let n = file.read(&mut buf).await?;
    let start = String::from_utf8_lossy(&buf[..n]);        

    if let Err(err) = flatterer_result {
        let mut res = Response::new(StatusCode::BadRequest);
        let output = json!({"id": id, "error": err.to_string(), "start": start});
        let body = Body::from_json(&output)?;
        res.set_body(body);
        return Ok(res);
    }

    let tmp_dir_path_to_move = tmp_dir_path.to_path_buf();

    let output_format = query.output_format.unwrap_or_else(|| "zip".to_string());

    if output_format == "fields" {
        let fields_value = fields_output(output_path.clone())?;
        let output = json!({"id": id, "fields": fields_value});
        let mut res = Response::new(StatusCode::Ok);
        let body = Body::from_json(&output)?;
        res.set_body(body);
        return Ok(res);
    }

    if output_format == "preview" {
        let fields_value = fields_output(output_path.clone())?;
        let preview_value = preview_output(output_path.clone(), fields_value).await?;
        let output = json!({"id": id, "preview": preview_value, "start": start});
        let mut res = Response::new(StatusCode::Ok);
        let body = Body::from_json(&output)?;
        res.set_body(body);
        return Ok(res);
    }

    if output_format == "xlsx" {
        let xlsx_file = File::open(output_path.join("output.xlsx")).await?;
        let xlsx_file_buf = BufReader::new(xlsx_file);

        let mut res = Response::new(StatusCode::Ok);
        let body = Body::from_reader(xlsx_file_buf, None);
        res.set_body(body);
        res.set_content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
        res.append_header(
            "Content-Disposition",
            format!("attachment; filename=\"{}.xlsx\"", "flatterer-output"),
        );
        return Ok(res);
    }

    if output_format == "csv" {
        let main_table_name = query.main_table_name.unwrap_or_else(|| "main".to_string());

        let csv_file = File::open(output_path.join(format!("csv/{}.csv", main_table_name))).await?;
        let csv_file_buf = BufReader::new(csv_file);

        let mut res = Response::new(StatusCode::Ok);
        let body = Body::from_reader(csv_file_buf, None);
        res.set_body(body);
        res.set_content_type("text/csv");
        res.append_header(
            "Content-Disposition",
            format!("attachment; filename=\"{}.csv\"", "flatterer-output"),
        );
        return Ok(res);
    }

    async_std::task::spawn_blocking(move || -> tide::Result<()> {
        zip_output(output_path.clone(), tmp_dir_path_to_move.to_path_buf())?;
        Ok(())
    })
    .await?;

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

fn run_flatterer(
    mut query: Query,
    download_file: String,
    output_path: PathBuf,
) -> tide::Result<()> {
    let file = StdFile::open(download_file)?;
    let reader = StdBufReader::new(file);

    let output_format = query.output_format.unwrap_or_else(|| "zip".to_string());

    if output_format != "zip" {
        query.csv = Some(false);
        query.xlsx = Some(false)
    }

    if output_format == "xlsx" {
        query.xlsx = Some(true)
    }
    if output_format == "csv" {
        query.csv = Some(true);
    }
    if output_format == "preview" {
        query.csv = Some(true);
    }

    let mut flat_files = FlatFiles::new(
        output_path.to_string_lossy().to_string(),
        query.csv.unwrap_or(true),
        query.xlsx.unwrap_or(false),
        true, // force
        query.main_table_name.unwrap_or_else(|| "main".to_string()),
        vec![], // list of json paths to omit object as if it was array
        query.inline_one_to_one.unwrap_or(false),
        query.json_schema.unwrap_or_else(|| "".to_string()),
        query.table_prefix.unwrap_or_else(|| "".to_string()),
        query.path_seperator.unwrap_or_else(|| "_".to_string()),
        query.schema_titles.unwrap_or_else(|| "".to_string()),
    )?;

    if output_format == "preview" {
        flat_files.preview = 10;
    }

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
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

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

fn fields_output(output_path: PathBuf) -> tide::Result<Vec<HashMap<String, String>>> {
    let mut csv_reader = Reader::from_path(output_path.join("fields.csv"))?;

    let mut all_fields = vec![];

    for result in csv_reader.deserialize() {
        let record: HashMap<String, String> = result?;
        all_fields.push(record)
    }
    Ok(all_fields)
}

async fn preview_output(output_path: PathBuf, fields: Vec<HashMap<String, String>>) -> tide::Result<Value> {
    let mut previews = vec![];

    let mut tables_reader = Reader::from_path(output_path.join("tables.csv"))?;

    for row in tables_reader.deserialize() {
        let table_row: HashMap<String, String> = row?;
        let table = table_row.get("table_name").unwrap().clone();
        let table_title = table_row.get("table_title").unwrap().clone();

        let path = output_path.join("csv").join(format!("{}.csv", table_title));

        let mut table_fields = vec![];

        for field in fields.iter() {
            if field.get("table_name").unwrap() == &table {
                table_fields.push(field.clone());
            }
        }

        let mut reader = Reader::from_path(path)?;
        for (row_num, row) in reader.deserialize().enumerate() {
            let row: Vec<String> = row?;
            for (col_num, item) in row.iter().enumerate(){
                table_fields[col_num].insert(format!("row {}", row_num), item.clone());
            }
        }

        let preview = json!({"table_name": table_title, "fields": table_fields});

        previews.push(preview);
    }
    Ok(serde_json::to_value(previews)?)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use async_std::fs::read_to_string;
    use tide_testing::TideTestingExt;

    #[test]
    fn test_preview_output() {
        async_std::task::block_on(async {
            let app = get_app().unwrap();

            let body_string = read_to_string("fixtures/basic.json").await.unwrap();

            let response_body: serde_json::value::Value = app
                .post("/api/convert?output_format=preview")
                .body(tide::Body::from_string(body_string))
                .content_type("application/json")
                .recv_json()
                .await
                .unwrap();

            insta::with_settings!({sort_maps => true}, {
                insta::assert_yaml_snapshot!(&response_body, {".id" => "[id]"});
            });
        })
    }
}
