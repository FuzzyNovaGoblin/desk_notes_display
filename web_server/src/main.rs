use axum::{Router, routing::get};
use regex::Regex;
use std::{fs, sync::OnceLock};

use crate::config::*;

mod config;

static TAB_REGEX: OnceLock<Regex> = OnceLock::<Regex>::new();
static TASK_REGEX: OnceLock<Regex> = OnceLock::<Regex>::new();
static HEADER_REGEX: OnceLock<Regex> = OnceLock::<Regex>::new();

#[tokio::main]
async fn main() {
    TAB_REGEX.get_or_init(|| Regex::new(r"\[([^\]]*)\]\([^\)]*\)").unwrap());
    TASK_REGEX.get_or_init(|| {
        Regex::new(r#"\s*(?:[-\+] (?<checkbox>\[(?<checked>.)])?)\s?(?<message>.*)"#).unwrap()
    });
    HEADER_REGEX
        .get_or_init(|| Regex::new(r#"(?<todo_header>^# TODO)\s*$|(^# [A-Za-z0-9\s]*$)"#).unwrap());

    let app = Router::new().route("/", get(handler));

    let listener = tokio::net::TcpListener::bind(SERVER_URL).await.unwrap();

    println!("listening on http://{}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

#[derive(PartialEq, Eq)]
enum UnderTodoCase {
    No,
    Yes,
    Past,
}

async fn handler() -> String {
    let mut under_todo_header = UnderTodoCase::No;

    fs::read_to_string(file_path())
        .unwrap()
        .replace("\x09", "  ")
        .lines()
        .map(|l| TAB_REGEX.get().unwrap().replace_all(l, "$1").to_string())
        .map(|l| {
            if l.len() <= 36 {
                l
            } else {
                l[0..36].to_owned()
            }
        })
        .filter(|l| {
            if under_todo_header == UnderTodoCase::Past {
                return false;
            }

            if HEADER_REGEX.get().unwrap().is_match(l) {
                if under_todo_header == UnderTodoCase::Yes {
                    under_todo_header = UnderTodoCase::Past;
                    false
                } else {
                    if HEADER_REGEX
                        .get()
                        .unwrap()
                        .captures(l)
                        .unwrap()
                        .name("todo_header")
                        .is_some()
                    {
                        under_todo_header = UnderTodoCase::Yes
                    }
                    true
                }
            } else if let Some(caps) = TASK_REGEX.get().unwrap().captures(l) {
                caps.name("checked")
                    .is_some_and(|c| c.as_str().trim().is_empty()) || caps.name("checked").is_none()
            } else {
                true
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
