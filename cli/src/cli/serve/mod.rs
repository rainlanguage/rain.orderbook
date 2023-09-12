mod serve_assets;
pub mod websocket;
use serve_assets::static_fie;
use actix_web::{web, App, HttpServer};
use clap::Parser;
use tracing::info;
use websocket::ws_index;
use crate::rpc::deposit::rpc_deposit;
use actix_cors::Cors;

#[derive(Parser,Debug,Clone)]
pub struct Serve {}

#[actix_web::main]
pub async fn handle_serve() -> std::io::Result<()> {
    let server = HttpServer::new(move || {

        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        
        App::new()
            .wrap(cors)
            .route("/ws/", web::get().to(ws_index))
            .service(rpc_deposit)
            .route("/{filename:.*}", web::get().to(static_fie))
    })
    .bind(("0.0.0.0", 8080))?
    .run();

    info!("Server running at http://localhost:8080/");
    server.await
}

