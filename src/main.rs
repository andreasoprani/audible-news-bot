use crate::storage::timestamp_log;
use aws_lambda_events::event::eventbridge::EventBridgeEvent;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use scraper::Html;
use serde::{Deserialize, Serialize};
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

pub mod book;
pub mod bot;
pub mod settings;
pub mod storage;
pub mod utils;

async fn get_html(url: &String) -> reqwest::Result<Html> {
    let response = reqwest::get(url.as_str()).await?;
    let html_content = response.text().await?;
    return Ok(Html::parse_document(&html_content));
}

async fn books_update(
    settings: &settings::Settings,
    tbot: &bot::TelegramBot,
    store: &Box<dyn storage::Storage>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut log_str = "".to_string();

    let document = match get_html(&settings.url).await {
        Ok(d) => d,
        Err(e) => {
            return Ok(timestamp_log(&format!("Error getting html: {}", e)));
        }
    };

    let books = match book::Book::from_html_document(document) {
        Ok(b) => b,
        Err(e) => {
            return Ok(timestamp_log(&format!("Error parsing html: {}", e)));
        }
    };

    let mut stored_books = match store.load_stored_books().await {
        Ok(b) => b,
        Err(e) => {
            return Ok(timestamp_log(&format!("Error loading books: {}", e)));
        }
    };

    let books_to_send = books
        .into_iter()
        .filter(|b| !stored_books.contains(b))
        .collect::<Vec<book::Book>>();

    for book in &books_to_send {
        log_str += timestamp_log(&book.formatted_log()).as_str();
        match tbot.send_book(book, &settings).await {
            Ok(_) => (),
            Err(e) => {
                log_str += timestamp_log(&format!("Error sending book: {}", e)).as_str();
                return Ok(log_str);
            }
        };
        stored_books.push(book.clone());
        stored_books = match store
            .store_books(stored_books, settings.max_books_kept)
            .await
        {
            Ok(b) => b,
            Err(e) => {
                log_str += timestamp_log(&format!("Error storing books: {}", e)).as_str();
                return Ok(log_str);
            }
        };
    }

    Ok(log_str)
}

async fn bot_fn() -> Result<(), Box<dyn std::error::Error>> {
    let store = storage::init_storage().await?;

    let mut log_str = timestamp_log("Startup.");

    let settings = store.load_settings().await?;

    let tbot = bot::TelegramBot::new(&settings)?;

    // TODO: telegram commands

    log_str += timestamp_log("Update begins.").as_str();
    log_str += &books_update(&settings, &tbot, &store).await?;
    log_str += timestamp_log(&"Update ends.").as_str();

    store.update_log(&log_str).await?;

    let _ = tbot.send_message_async(log_str, true).await?;

    return Ok(());
}

#[derive(Serialize, Deserialize)]
struct TriggerEvent {}

async fn function_handler(
    _event: LambdaEvent<EventBridgeEvent<TriggerEvent>>,
) -> Result<(), Error> {
    match bot_fn().await {
        Ok(_) => Ok(()),
        Err(e) => {
            let error_string = format!("BOT ERROR: {}", e);
            tracing::error!("{}", error_string.as_str());
            Err(error_string.into())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
