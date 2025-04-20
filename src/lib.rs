use actix_web::{web::{self}, App, Either, HttpResponse, HttpServer, Responder, Result};
use futures::StreamExt;
use actix_web::body::BoxBody;
use csv::Reader;
use actix_files::Files;
use std::fs::File;
use std::{collections::HashMap, env::var};
use walkdir::WalkDir;
use actix_web::middleware::Logger;
use serde::Deserialize;
use tempfile::TempDir;
use uuid::Uuid;
use serde_json::{json, Value};
use std::path::PathBuf;
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::MultipartForm;
use std::io::Read;
use libflatterer::{flatten, Options};


#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(limit = "100MB")]
    file: Option<TempFile>,
    #[multipart(limit = "100MB")]
    fields: Option<TempFile>,
    #[multipart(limit = "100MB")]
    tables: Option<TempFile>,
}

#[derive(Deserialize, Debug, Clone)]
struct Query {
    id: Option<String>,
    output_format: Option<String>,
    file_url: Option<String>,
    array_key: Option<String>,
    json_lines: Option<bool>,
    main_table_name: Option<String>,
    inline_one_to_one: Option<bool>,
    json_schema: Option<String>,
    table_prefix: Option<String>,
    path_separator: Option<String>,
    schema_titles: Option<String>,
    fields_only: Option<bool>,
    tables_only: Option<bool>,
    pushdown: Option<String>,
}

fn run_flatterer(
    query: Query,
    download_path: PathBuf,
    output_path: PathBuf,
    json_lines: bool,
    path: String,
) -> Result<()> {
    let file = std::fs::File::open(download_path.join("download.json"))?;
    let reader = std::io::BufReader::new(file);

    let output_format = query.output_format.unwrap_or_else(|| "zip".to_string());

    let mut options = Options::builder().build();

    if output_format != "zip" {
        options.csv = false;
        options.xlsx = false;
        options.sqlite = false;
    }

    if output_format == "xlsx" {
        options.xlsx = true;
    }
    if output_format == "csv" {
        options.csv = true;
    }
    if output_format == "sqlite" {
        options.sqlite = true;
    }
    if output_format == "preview" {
        options.csv = true;
        options.preview = 10;
    }
    options.force = true;
    options.main_table_name = query.main_table_name.unwrap_or_else(|| "main".to_string());

    options.inline_one_to_one = query.inline_one_to_one.unwrap_or(false);

    options.schema = query.json_schema.unwrap_or_else(|| "".to_string());

    options.table_prefix = query.table_prefix.unwrap_or_else(|| "".to_string());
    options.path_separator = query.path_separator.unwrap_or_else(|| "_".to_string());
    options.schema_titles = query.schema_titles.unwrap_or_else(|| "".to_string());
    options.json_stream = json_lines;

    let fields_path = download_path.join("fields.csv");
    if fields_path.exists() {
        options.fields_csv = fields_path.to_string_lossy().into();
    }
    options.only_fields = query.fields_only.unwrap_or_else(|| false);

    let tables_path = download_path.join("tables.csv");
    if tables_path.exists() {
        options.tables_csv = tables_path.to_string_lossy().into();
    }
    options.only_tables = query.tables_only.unwrap_or_else(|| false);

    let pushdown = query.pushdown.unwrap_or_else(|| "".into());
    if !pushdown.is_empty() {
        options.pushdown = vec![pushdown];
    }

    let mut path_vec = vec![];

    if !path.is_empty() && !json_lines {
        path_vec.push(path);
    }
    options.path = path_vec;

    flatten(
        Box::new(reader),
        output_path.to_string_lossy().to_string(),
        options
    ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(())
}

async fn download(url_string: String, tmp_dir: PathBuf) -> eyre::Result<()> {

    if !url_string.starts_with("http") {
        // return Err(tide::Error::from_str(tide::StatusCode::BadRequest, "`url` is empty or does not start with `http`"))
        return Err(eyre::eyre!("`url` is empty or does not start with `http`"))
    }
    let download_file = tmp_dir.join("download.json");

    let mut file = tokio::fs::File::create(&download_file).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let mut stream = reqwest::get(&url_string).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?.bytes_stream();

    while let Some(item) = stream.next().await {
        tokio::io::copy(&mut item?.as_ref(), &mut file).await?;
    }

    Ok(())
}

fn fields_output(output_path: PathBuf) -> csv::Result<Vec<HashMap<String, String>>> {
    let mut csv_reader = Reader::from_path(output_path.join("fields.csv"))?;

    let mut all_fields = vec![];

    for result in csv_reader.deserialize() {
        let record: HashMap<String, String> = result?;
        all_fields.push(record)
    }
    Ok(all_fields)
}

async fn preview_output(output_path: PathBuf, fields: Vec<HashMap<String, String>>) -> csv::Result<Value> {
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
    Ok(serde_json::to_value(previews).expect("should not have issue converting to value"))
}

fn zip_output(output_path: PathBuf, tmp_dir_path: PathBuf) -> Result<()> {
    let zip_file = tmp_dir_path.join("export.zip");

    let file = File::create(&zip_file)?;
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
                path.strip_prefix(output_path.clone()).expect("known to be a directory").to_string_lossy(),
                options,
            ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        } else {
            zip.start_file(
                path.strip_prefix(output_path.clone()).expect("known to be a file").to_string_lossy(),
                options,
            ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            let mut file = File::open(path)?;
            std::io::copy(&mut file, &mut zip)?;
        }
    }
    Ok(())
}


fn internal_error_json(error: String) -> HttpResponse<BoxBody> {
    HttpResponse::InternalServerError().body(json!({"error": error}).to_string())
}

fn bad_request_json(error_json: Value) -> HttpResponse<BoxBody> {
    HttpResponse::BadRequest().body(error_json.to_string())
}

async fn convert(query: web::Query<Query>) -> Either<HttpResponse<BoxBody>, impl Responder> {
    process(query, None).await
}

async fn get_input(query: web::Query<Query>, MultipartForm(form): MultipartForm<UploadForm>) -> Either<HttpResponse<BoxBody>, impl Responder> {
    process(query, Some(form)).await
}

async fn wasm() -> impl Responder {
    return HttpResponse::Ok().body(json!({"wasm": false}).to_string());
}

async fn process(query: web::Query<Query>, upload_form: Option<UploadForm>) -> Either<HttpResponse<BoxBody>, impl Responder> {
    let tmp_dir = TempDir::new();
    if let Err(e) = tmp_dir {
        return Either::Left(internal_error_json(format!("Error creating temp dir: {:?}", e)));
    } 
    let tmp_dir_path = tmp_dir.expect("just checked").path().to_owned();

    let output_path = tmp_dir_path.join("output");

    let mut json_output;

    if let Some(id) = &query.id {
        json_output = json!({ "id": id });
    } else {  
        let mut uploaded_files = vec![];
        let clean_tmp_result = clean_tmp();
        if let Err(e) = clean_tmp_result {
            return Either::Left(internal_error_json(format!("Error cleaning tmp dir: {:?}", e)));
        }
        let uuid = Uuid::new_v4().hyphenated();
        let tmp_dir = std::env::temp_dir().join(format!("flatterer-{}", uuid));
        json_output = json!({ "id": uuid.to_string() });
        let create_dir_result = std::fs::create_dir(&tmp_dir);
        if let Err(e) = create_dir_result {
            return Either::Left(internal_error_json(format!("Error creating tmp dir: {:?}", e)));
        }

        if let Some(form) = upload_form {
            if form.file.is_some() {
                let file = form.file.unwrap();
                let file_parsist_result = file.file.persist(tmp_dir.join("download.json"));
                if let Err(e) = file_parsist_result {
                    return Either::Left(internal_error_json(format!("Error persisting file: {:?}", e)));
                }
                uploaded_files.push("file".to_string());
            }

            if form.fields.is_some() {
                let fields = form.fields.unwrap();
                let file_parsist_result = fields.file.persist(tmp_dir.join("fields.csv"));
                if let Err(e) = file_parsist_result {
                    return Either::Left(internal_error_json(format!("Error persisting file: {:?}", e)));
                }
                uploaded_files.push("fields".to_string());
            }

            if form.tables.is_some() {
                let tables = form.tables.unwrap();
                let file_parsist_result = tables.file.persist(tmp_dir.join("tables.csv"));
                if let Err(e) = file_parsist_result {
                    return Either::Left(internal_error_json(format!("Error persisting file: {:?}", e)));
                }
                uploaded_files.push("tables".to_string());
            }

            if let Some(file_url) = &query.file_url {
                if let Err(error) = download(file_url.clone(), tmp_dir).await {
                    json_output = json!({"error": error.to_string()})
                }
                uploaded_files.push("file".to_string());
            }
        }

        if !uploaded_files.contains(&"file".to_string()) {
            json_output = json!({"error": "need to supply either an id or filename or supply data in request body"});
        }
    }

    let mut download_path = std::env::temp_dir();
    let mut download_file = std::env::temp_dir();
    let mut id = "".to_string();

    if let Some(id_value) = json_output.get("id") {
        if let Some(id_string) = id_value.as_str() {
            id = id_string.to_string();
            download_path.push(format!("flatterer-{}", id_string));
            download_file.push(format!("flatterer-{}", id_string));
            download_file.push("download.json");
            if !download_file.exists() {
                json_output = json!({"error": "id does not exist, you may need to ask you file to be downloaded again or to upload the file again."})
            }
        }
    }

    if json_output.get("error").is_some() {
        // return Either::Left(generate_json_error(format!("Error creating temp dir")));
        return Either::Left(bad_request_json(json_output))
    }

    let file_result = File::open(download_file);
    if file_result.is_err() {
        return Either::Left(internal_error_json(format!("Error opening file: {:?}", file_result.err().unwrap())));
    }
    let mut file = file_result.unwrap();

    let mut buf = vec![0;10240];
    let read_result = file.read(&mut buf);
    if read_result.is_err() {
        return Either::Left(internal_error_json(format!("Error reading file: {:?}", read_result.err().unwrap())));
    }
    let n = read_result.unwrap();
    let start = String::from_utf8_lossy(&buf[..n]);        

    let mut path = "".to_string();

    if let Some(array_key) = &query.array_key {
        path = array_key.to_owned();
    };

    let mut json_lines = query.json_lines.unwrap_or(false);

    let mut guess_text = "".to_string();

    if path.is_empty() && !json_lines {
        match libflatterer::guess_array(&start) {
            Ok((guess, _)) => {
                if guess == "stream" {
                    json_lines = true;
                    guess_text = "JSON Stream".to_string()
                };
            }
            Err(err) => {
                let output = json!({"id": id, "error": err.to_string(), "start": start});
                return Either::Left(bad_request_json(output))
            }
        }
    }

    let output_path_copy = output_path.clone();
    let query_copy = query.clone().into_inner();

    if let Err(err) = run_flatterer(query_copy, download_path, output_path_copy, json_lines, path) {
        let output = json!({"id": id, "error": err.to_string(), "start": start});
        return Either::Left(bad_request_json(output))
    }

    let tmp_dir_path_to_move = tmp_dir_path.to_path_buf();

    let output_format = query.output_format.clone().unwrap_or_else(|| "zip".to_string());

    if output_format == "fields" {
        return Either::Right(actix_files::NamedFile::open_async(output_path.join("fields.csv")).await);
    }

    if output_format == "tables" {
        return Either::Right(actix_files::NamedFile::open_async(output_path.join("tables.csv")).await);
    }

    if output_format == "preview" {
        let fields_value_result = fields_output(output_path.clone());
        if let Err(e) = fields_value_result {
            return Either::Left(internal_error_json(format!("Error reading fields.csv: {:?}", e)));
        }
        let fields_value = fields_value_result.unwrap();

        let preview_value_result = preview_output(output_path.clone(), fields_value).await;
        if let Err(e) = preview_value_result {
            return Either::Left(internal_error_json(format!("Error creating preview: {:?}", e)));
        }

        let preview_value = preview_value_result.expect("just checked");
        let output = json!({"id": id, "preview": preview_value, "start": start, "guess_text": guess_text});

        return Either::Left(HttpResponse::Ok().body(output.to_string()));

    }

    if output_format == "xlsx" {
        return Either::Right(actix_files::NamedFile::open_async(output_path.join("output.xlsx")).await);
    }
    
    if output_format == "sqlite" {
        return Either::Right(actix_files::NamedFile::open_async(output_path.join("sqlite.db")).await);
    }

    if output_format == "csv" {
        let main_table_name = query.main_table_name.clone().unwrap_or_else(|| "main".to_string());
        return Either::Right(actix_files::NamedFile::open_async(output_path.join("csv").join(format!("{}.csv", main_table_name))).await);
    }

    let zip_output_result = zip_output(output_path.clone(), tmp_dir_path_to_move.to_path_buf());

    if let Err(e) = zip_output_result {
        return Either::Left(internal_error_json(format!("Error zipping output: {:?}", e)));
    }

    return Either::Right(actix_files::NamedFile::open_async( tmp_dir_path.join("export.zip")).await);
}


fn clean_tmp() ->  std::io::Result<()> {

    let clean_tmp_time = if let Ok(clean_tmp_time) = var("CLEAN_TMP_TIME") {
        match clean_tmp_time.parse::<u64>() {
            Ok(clean_tmp_time) => {clean_tmp_time},
            _ => {3600}
        }
    } else {
        3600
    };


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
        if entry.metadata()?.modified()?.elapsed().map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "elapsed time not able to be calculated"))?.as_secs() > clean_tmp_time {
            log::debug!("Removing tmp dir: {:?}", entry);

            if entry.metadata()?.is_dir() {
                std::fs::remove_dir_all(&entry.into_path())?;
            }
        }
    }
    Ok(())
}


#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    env_logger::init();
    clean_tmp()?;

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



    let open_browser = if let Ok(_) = var("OPEN_BROWSER") {
        true
    } else {
        false
    };

    let path = format!("http://{}:{}", host, port);

    if open_browser {
        match open::that(&path) {
            Ok(()) => println!("Opened browser '{}' successfully.", path),
            Err(err) => eprintln!("An error occurred when opening browser'{}': {}", path, err),
        } 
    } else {
        println!("Running at '{path}'.")
    }

    let static_files = if let Ok(static_files) = var("STATIC_FILES") {
        if let Some(static_files) = static_files.strip_suffix("/") {
            static_files.to_owned()
        } else {
            static_files
        }
    } else {
        "dist".to_owned()
    };

    let port: u16 = port.parse().expect("PORT must be a valid u16 integer");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(
                web::resource("/wasm.json").route(web::get().to(wasm))
            )
            .service(
                web::resource("/api/get_input")
                .route(web::post().to(get_input))
                .route(web::get().to(get_input))
                .route(web::put().to(get_input))
            )
            .service(
                web::resource("/api/convert")
                .route(web::post().to(convert))
                .route(web::get().to(convert))
                .route(web::put().to(convert))
            )
            .service(Files::new("/", static_files.clone()).index_file("index.html"))
    })
    .bind((host, port))?
    .run()
    .await
}