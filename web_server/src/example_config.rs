pub const SERVER_URL:&str = "0.0.0.0:7272";

pub fn file_path() -> String{
    format!(
        "{}/obsidian/vault/path/TODO.md",
        env!("HOME")
    )
}