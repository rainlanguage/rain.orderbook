use crate::serve_assets::static_fie;
use actix_web::http::header;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};

mod serve_assets;

#[actix_web::main]
async fn handle_serve() -> std::io::Result<()> {
    let server = HttpServer::new(|| {
        App::new()
            .service(hello)
            .route("/{filename:.*}", web::get().to(static_fie))
    })
    .bind(("0.0.0.0", 8080))?
    .run();
    println!("Started");
    server.await
}
