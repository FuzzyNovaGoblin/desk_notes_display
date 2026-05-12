use axum::{Router, routing::get};
use regex::Regex;
use std::{fs, sync::OnceLock};

use crate::config::*;

mod config;

static TAB_REGEX: OnceLock<Regex> = OnceLock::<Regex>::new();
static TASK_REGEX: OnceLock<Regex> = OnceLock::<Regex>::new();
static HEADER_REGEX: OnceLock<Regex> = OnceLock::<Regex>::new();
static BULLET_REGEX: OnceLock<Regex> = OnceLock::<Regex>::new();

#[tokio::main]
async fn main() {
    TAB_REGEX.get_or_init(|| Regex::new(r"\[([^\]]*)\]\([^\)]*\)").unwrap());
    TASK_REGEX.get_or_init(|| {
        Regex::new(r#"\s*(?:[-\+] (?<checkbox>\[(?<checked>.)])?)\s?(?<message>.*)"#).unwrap()
    });
    HEADER_REGEX
        .get_or_init(|| Regex::new(r#"(?<todo_header>^# TODO)\s*$|(^# [A-Za-z0-9\s]*$)"#).unwrap());
    BULLET_REGEX.get_or_init(|| Regex::new(r"(?<indent>\s*)[-\+] (?<checkbox>\[.\] )?").unwrap());

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
    let mut items_before_todo = false;

    fs::read_to_string(file_path())
        .unwrap()
        .replace("\x09", "  ")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| TAB_REGEX.get().unwrap().replace_all(l, "$1").to_string())
        .filter(|l| {
            if under_todo_header == UnderTodoCase::Past {
                return false;
            }

            if HEADER_REGEX.get().unwrap().is_match(l) {
                if under_todo_header == UnderTodoCase::Yes {
                    under_todo_header = UnderTodoCase::Past;
                    false
                } else if HEADER_REGEX
                    .get()
                    .unwrap()
                    .captures(l)
                    .unwrap()
                    .name("todo_header")
                    .is_some()
                {
                    under_todo_header = UnderTodoCase::Yes;

                    items_before_todo
                } else {
                    true
                }
            } else if let Some(caps) = TASK_REGEX.get().unwrap().captures(l) {
                let t = caps
                    .name("checked")
                    .is_some_and(|c| c.as_str().trim().is_empty())
                    || caps.name("checked").is_none();
                if !items_before_todo {
                    items_before_todo = t;
                }
                t
            } else {
                if !items_before_todo && under_todo_header == UnderTodoCase::No {
                    items_before_todo = true
                }
                true
            }
        })
        .map(|line| {
            let caps = match BULLET_REGEX.get().unwrap().captures(&line) {
                Some(v) => v,
                None => return line,
            };
            let spaces = match caps.name("indent") {
                Some(v) => v.len() / 2,
                None => 0,
            };
            let bullet = match caps.name("checkbox") {
                // Some(_) => '\u{0220}',
                // Some(_) => '\u{0254}',
                // Some(_) => '\u{007F}',
                // Some(_) => '\u{25CB}',
                Some(_) => "[]",
                None => "-",
            };
            BULLET_REGEX
                .get()
                .unwrap()
                .replace(
                    &line,
                    format!("{}{bullet}", (0..spaces).map(|_| ' ').collect::<String>()),
                )
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}
