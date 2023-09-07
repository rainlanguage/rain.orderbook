mod serve_assets;
use crate::cli::serve::serve_assets::static_fie;
use actix_web::http::header;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use clap::Parser;

#[derive(Parser,Debug,Clone)]
pub struct Serve{
}  

#[actix_web::main]
pub async fn handle_serve() -> std::io::Result<()> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/{filename:.*}", web::get().to(static_fie))
    })
    .bind(("0.0.0.0", 8080))?
    .run();
    println!("Started");
    server.await
}
