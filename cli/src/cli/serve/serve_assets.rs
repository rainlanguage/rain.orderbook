use actix_web::http::header;
use actix_web::{HttpRequest, HttpResponse, Responder};
use once_cell::unsync::Lazy;
use std::path::PathBuf;
include!(concat!(env!("OUT_DIR"), "/generated.rs"));

pub async fn static_fie(req: HttpRequest) -> impl Responder {
    let assets = Lazy::new(|| {
        generate()
    });

    let path = req.match_info().query("filename").parse::<PathBuf>().unwrap();
    let path = path.to_str();
    let path = if let Some(path) = path { path } else {return HttpResponse::BadRequest().finish() };
    let path = if path.is_empty() { "index.html" } else { path };
    let static_file = assets.get(path);
    match static_file {
        Some(file) => {
            let file: &static_files::Resource = file;
            println!("Loading file {:?}", file.mime_type);
            HttpResponse::Ok()
                .insert_header((header::CONTENT_TYPE, file.mime_type))
                .body(file.data)
        }
        None => HttpResponse::NotFound().finish(),
    }
}
