//! Inplementation of a bitcoin-like system

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

///
#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("<h1>Hello, World</h1>")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    const ADDRESS: &str = "127.0.0.1";
    const PORT: u16 = 7890;

    HttpServer::new(|| {
        App::new()
            .service(hello)
    })
    .bind((ADDRESS, PORT))?
    .run()
    .await
}