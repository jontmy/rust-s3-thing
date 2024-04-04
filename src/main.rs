extern crate dotenv;

use axum::{routing::get, Router};
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));
    let port = env::var("PORT").unwrap();
    let addr = format!("localhost:{port}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
