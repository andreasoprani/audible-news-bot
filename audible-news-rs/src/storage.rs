use crate::book::Book;
use crate::settings::Settings;
use async_trait::async_trait;

pub mod s3_storage;

#[async_trait]
pub trait Storage {
    async fn load_settings(&self) -> Result<Settings, Box<dyn std::error::Error>>;

    async fn update_log(&self, log_line: &str) -> Result<String, Box<dyn std::error::Error>>;

    async fn load_stored_books(&self) -> Result<Vec<Book>, Box<dyn std::error::Error>>;

    async fn store_books(
        &self,
        books: Vec<Book>,
        max_books: u32,
    ) -> Result<Vec<Book>, Box<dyn std::error::Error>>;
}

pub fn timestamp_log(log_line: &str) -> String {
    format!(
        "{}: {}\n",
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        log_line
    )
}

pub async fn init_storage() -> Result<Box<dyn Storage>, Box<dyn std::error::Error>> {
    let storage_type =
        std::env::var("STORAGE_TYPE").expect("Missing environment variable: STORAGE_TYPE");
    match storage_type.as_str() {
        "S3" => Ok(Box::new(s3_storage::S3Storage::new().await)),
        _ => Err(format!("Invalid storage type {}", storage_type).into()),
    }
}
