use std::fs;

use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7272").await.unwrap();

    println!("listening on http://{}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> String {
    fs::read_to_string(format!(
        "{}/obsidian/fuz-vault/General/TODO.md",
        env!("HOME")
    ))
    .unwrap()
    .replace("\x09", "  ")
    .lines()
    .map(|l| if l.len() <= 36 { &l } else { &l[0..36] })
    .collect::<Vec<_>>()
    .join("\n")
}
