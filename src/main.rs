//! Inplementation of a bitcoin-like system
use axum::routing::get;
use axum::Router;
use std::net::SocketAddr;
use std::option::Option;

#[tokio::main]
async fn main() {
    const ROOT_NODE_ADDR: Option<String> = None;

    match ROOT_NODE_ADDR {
        Some(addr) => {
            println!("Requesting {addr} for peers addresses");
        }
        None => {
            println!("This node will run as the root node");
        }
    }

    let app = Router::new().route("/ping", get(pong));

    let server = axum::Server::bind(&"0.0.0.0:0".parse().unwrap()).serve(app.into_make_service());

    if let SocketAddr::V4(addr4) = server.local_addr() {
        let port = addr4.port();
        let ip = addr4.ip();
        println!("Listening on http://{ip}:{port}");
        println!("Listening on localhost http://127.0.0.1:{port}");
        println!("use ping-pong to test the server: http://127.0.0.1:{port}/ping");
        println!("you should get a \"pong\" in response");
    }

    server.await.unwrap();
}

///
async fn pong() -> &'static str {
    "pong"
}

#[allow(dead_code)]
async fn ping(addr: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("http://{addr}/ping");
    let response = reqwest::get(url).await?.text().await?;
    println!("{response}");
    Ok(response)
}
