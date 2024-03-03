use crate::book::Book;
use crate::settings::Settings;
use crate::storage::{timestamp_log, Storage};
use async_trait::async_trait;

use aws_config::{meta::region::RegionProviderChain, BehaviorVersion, Region};

pub struct S3Storage {
    pub client: aws_sdk_s3::Client,
    pub bucket: String,
}

impl S3Storage {
    pub async fn new() -> S3Storage {
        let region_provider =
            RegionProviderChain::first_try(std::env::var("S3_REGION").ok().map(Region::new))
                .or_default_provider()
                .or_else(Region::new("eu-central-1"));
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;
        S3Storage {
            client: aws_sdk_s3::Client::new(&config),
            bucket: std::env::var("S3_BUCKET").expect("Missing environment variable: S3_BUCKET"),
        }
    }

    async fn read_string_from_s3(
        &self,
        filename: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let object = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(filename)
            .send()
            .await?;
        let bytes = object.body.collect().await?.into_bytes();
        let json_str = std::str::from_utf8(&bytes)?.to_owned();
        Ok(json_str)
    }

    async fn write_string_to_s3(
        &self,
        filename: &str,
        string: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(filename)
            .body(aws_sdk_s3::primitives::ByteStream::from(
                String::from(string).into_bytes(),
            ))
            .send()
            .await?;
        Ok(())
    }

    async fn append_string_to_s3(
        &self,
        filename: &str,
        string: &str,
        max_lines: Option<u32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut content = self.read_string_from_s3(filename).await?;
        content.push_str(string);
        match max_lines {
            Some(max) => {
                let lines = content.lines();
                if lines.clone().count() > max as usize {
                    content = lines
                        .rev()
                        .take(max as usize)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect::<Vec<_>>()
                        .join("\n");
                }
            }
            None => (),
        }
        let _ = self.write_string_to_s3(filename, &content).await?;
        Ok(())
    }
}

#[async_trait]
impl Storage for S3Storage {
    async fn load_settings(&self) -> Result<Settings, Box<dyn std::error::Error>> {
        let settings_file = std::env::var("SETTINGS_JSON_FILE")
            .expect("Missing environment variable: SETTINGS_JSON_FILE");
        let json_string = self.read_string_from_s3(settings_file.as_str()).await?;
        let settings = serde_json::from_str(json_string.as_str())?;
        Ok(settings)
    }

    async fn update_log(&self, log_line: &str) -> Result<String, Box<dyn std::error::Error>> {
        let log_file =
            std::env::var("LOG_TXT_FILE").expect("Missing environment variable: LOG_TXT_FILE");
        let timestamped_str = timestamp_log(log_line);
        let _ = self
            .append_string_to_s3(log_file.as_str(), &timestamped_str.as_str(), None)
            .await?;
        Ok(timestamped_str)
    }

    async fn load_stored_books(&self) -> Result<Vec<Book>, Box<dyn std::error::Error>> {
        let book_file = std::env::var("BOOKS_JSON_FILE")
            .expect("Missing environment variable: BOOKS_JSON_FILE");
        let json_string = self.read_string_from_s3(book_file.as_str()).await?;
        let books: Vec<Book> = serde_json::from_str(json_string.as_str())?;
        Ok(books)
    }

    async fn store_books(
        &self,
        mut books: Vec<Book>,
        max_books: u32,
    ) -> Result<Vec<Book>, Box<dyn std::error::Error>> {
        books = Book::limit(books, max_books);
        let book_file = std::env::var("BOOKS_JSON_FILE")
            .expect("Missing environment variable: BOOKS_JSON_FILE");
        let json_string = serde_json::to_string_pretty(&books);
        self.write_string_to_s3(book_file.as_str(), &json_string?)
            .await?;
        Ok(books)
    }
}
