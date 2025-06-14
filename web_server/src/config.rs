pub const SERVER_URL:&str = "0.0.0.0:7272";

pub fn file_path() -> String{
    format!(
        "{}/obsidian/fuz-vault/General/TODO.md",
        env!("HOME")
    )
}