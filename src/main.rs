//! Inplementation of a bitcoin-like system

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
// install the redis server first
// https://redis.io/topics/quickstart
use redis::{Client, Commands, Connection, RedisError};
// install the mongodb server first
// https://www.mongodb.com/developer/languages/rust/rust-mongodb-crud-tutorial/
use mongodb::{Collection, Database, bson::doc, error::Error, options::ClientOptions};

#[get("/")]
async fn hello(req_body: String) -> impl Responder {
    println!("{}", req_body);
    HttpResponse::Ok().body("<h1>Hello, World</h1>")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    const ADDRESS: &str = "127.0.0.1";
    const PORT: u16 = 7890;

    println!("Server is running on http://{}:{}", ADDRESS, PORT);

    HttpServer::new(|| {
        App::new()
            .service(hello)
    })
        .bind((ADDRESS, PORT))?
        .run()
        .await
}