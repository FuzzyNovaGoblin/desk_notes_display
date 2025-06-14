use std::fs;
use axum::{Router, routing::get};

use crate::config::*;

mod config;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handler));

    let listener = tokio::net::TcpListener::bind(SERVER_URL).await.unwrap();

    println!("listening on http://{}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> String {
    fs::read_to_string(file_path())
    .unwrap()
    .replace("\x09", "  ")
    .lines()
    .map(|l| if l.len() <= 36 { &l } else { &l[0..36] })
    .collect::<Vec<_>>()
    .join("\n")
}
