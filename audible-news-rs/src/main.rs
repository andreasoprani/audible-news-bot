use clokwerk::TimeUnits;
use scraper::Html;
use std::io::Write;
use teloxide::{
    requests::RequesterExt,
    types::{ChatId, ParseMode},
    Bot,
};

pub mod book;
pub mod bot;
pub mod settings;
pub mod utils;

static LOG_FILE: &str = "data/log.txt";
static SETTINGS_FILE: &str = "data/bot_settings.json";
static BOOKS_FILE: &str = "data/books.json";

fn update_log_file(str_to_append: &String) -> String {
    let mut timestamped_str = format!(
        "{} - {}\n",
        chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        str_to_append
    );
    match std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(LOG_FILE)
    {
        Ok(mut f) => match write!(f, "{}", timestamped_str) {
            Err(e) => {
                eprintln!("Couldn't write to file: {}", e);
                timestamped_str += &format!(" - Error writing to log: {}", e);
            }
            _ => (),
        },
        Err(e) => {
            eprintln!("Couldn't open file: {}", e);
            timestamped_str += &format!(" - Error opening log file: {}", e);
        }
    };
    timestamped_str
}

fn get_html(url: &String) -> reqwest::Result<Html> {
    let response = reqwest::blocking::get(url);
    let html_content = match response {
        Ok(r) => match r.text() {
            Ok(t) => t,
            Err(e) => return Err(e),
        },
        Err(e) => return Err(e),
    };
    return Ok(Html::parse_document(&html_content));
}

fn books_update(settings: settings::Settings, tbot: &bot::TelegramBot) -> String {
    let mut log_str = "".to_string();

    let document = match get_html(&settings.url) {
        Ok(d) => d,
        Err(e) => {
            return update_log_file(&format!("Error getting html: {}", e));
        }
    };

    let books = match book::Book::from_html_document(document) {
        Ok(b) => b,
        Err(e) => {
            return update_log_file(&format!("Error parsing html: {}", e));
        }
    };

    let mut stored_books = match book::Book::load_from_json(BOOKS_FILE) {
        Ok(b) => b,
        Err(e) => {
            return update_log_file(&format!("Error loading books: {}", e));
        }
    };
    let books_to_send = books
        .into_iter()
        .filter(|b| !stored_books.contains(b))
        .collect::<Vec<book::Book>>();

    for book in &books_to_send {
        log_str += &update_log_file(&book.formatted_log());
        match tbot.send_book(book, &settings) {
            Ok(_) => (),
            Err(e) => {
                log_str += &update_log_file(&format!("Error sending book: {}", e));
                return log_str;
            }
        };
        stored_books.push(book.clone());
        stored_books =
            match book::Book::store_to_json(stored_books, BOOKS_FILE, settings.max_books_kept) {
                Ok(b) => b,
                Err(e) => {
                    log_str += &update_log_file(&format!("Error storing books: {}", e));
                    return log_str;
                }
            };
    }

    log_str
}

fn bot_fn() -> Result<(), Box<dyn std::error::Error>> {
    let mut log_str = update_log_file(&"Startup.".to_string());

    let settings = settings::Settings::from_file(SETTINGS_FILE)?;

    let tbot = bot::TelegramBot {
        bot: Bot::from_env().parse_mode(ParseMode::MarkdownV2), // Needs TELOXIDE_TOKEN env variable
        channel_id: ChatId(
            std::env::var("CHANNEL_ID")
                .expect("CHANNEL_ID must be present")
                .parse()?,
        ),
        admin_chat_id: ChatId(
            std::env::var("ADMIN_CHAT_ID")
                .expect("ADMIN_CHAT_ID must be present")
                .parse()?,
        ),
        max_message_length: settings.max_message_length,
    };

    // TODO: telegram commands

    log_str += &update_log_file(&"Update begins.".to_string());
    log_str += &books_update(settings, &tbot);
    log_str += &update_log_file(&"Update ends.".to_string());

    tbot.send_message_blocking(log_str, true)?;

    return Ok(());
}

fn get_schedule_interval() -> clokwerk::Interval {
    match std::env::var("MINUTES") {
        Ok(s) => s.parse::<u32>().unwrap().minutes(),
        Err(_) => match std::env::var("HOURS") {
            Ok(s) => s.parse::<u32>().unwrap().hours(),
            Err(_) => match std::env::var("DAYS") {
                Ok(s) => s.parse::<u32>().unwrap().days(),
                Err(_) => 1.hours(),
            },
        },
    }
}

fn main() {
    let mut scheduler = clokwerk::Scheduler::new();
    let interval = get_schedule_interval();
    println!("Scheduling update every {:?}.", interval);
    scheduler.every(interval).run(|| {
        println!("Running update.");
        match bot_fn() {
            Ok(_) => (),
            Err(e) => {
                update_log_file(&format!("Error in update functions: {}", e));
            }
        }
    });

    loop {
        scheduler.run_pending();
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}
