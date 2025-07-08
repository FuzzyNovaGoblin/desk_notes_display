use axum::{Router, routing::get};
use regex::Regex;
use std::fs;

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
    let re = Regex::new(r"\[([^\]]*)\]\([^\)]*\)").unwrap();

    fs::read_to_string(file_path())
        .unwrap()
        .replace("\x09", "  ")
        .lines()
        .map(|l| re.replace_all(l, "$1").to_string())
        .map(|l| {
            if l.len() <= 36 {
                l
            } else {
                l[0..36].to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
