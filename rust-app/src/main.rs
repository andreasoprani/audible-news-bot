use chrono;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use teloxide::Bot;
use url::Url;

static LOG_FILE: &str = "data/log.txt";
static SETTINGS_FILE: &str = "data/bot_settings.json";
static BOT_STATE_FILE: &str = "data/bot_state.json";
static BOOKS_FILE: &str = "data/books.json";

#[derive(Debug, Deserialize)]
struct AttributesMap {
    title: String,
    author: String,
    narrator: String,
    runtime: String,
    date: String,
}

#[derive(Debug, Deserialize)]
struct Settings {
    url: String,
    url_header: String,
    max_books_kept: u32,
    max_message_length: u32,
    default_log_length: u32,
    allowed_commands: Vec<String>,
    attribute_names: AttributesMap,
    redirect_message: String,
    book_url_message: String,
}

impl Settings {
    fn from_file(filename: &str) -> Self {
        let json = std::fs::read_to_string(filename).unwrap();
        serde_json::from_str(&json).unwrap()
    }
}

#[derive(Debug, Deserialize)]
struct BotState {
    active: bool,
    last_update: u64,
}

impl BotState {
    fn from_file(filename: &str) -> Self {
        let json = std::fs::read_to_string(filename).unwrap();
        serde_json::from_str(&json).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Book {
    title: String,
    author: Option<String>,
    narrator: Option<String>,
    runtime: String,
    date: String,
    url: String,
}

impl Book {
    fn from_html_document(document: Html) -> Vec<Book> {
        fn get_text(
            product: scraper::element_ref::ElementRef,
            selector: &str,
            optional: bool,
            remove: &str,
        ) -> Option<String> {
            match product
                .select(&Selector::parse(selector).unwrap())
                .next()
                .map(|p| p.text().collect::<String>())
            {
                Some(s) => Some(s.replace(remove, "").trim().to_string()),
                None => {
                    if optional {
                        None
                    } else {
                        panic!("{} not found", selector)
                    }
                }
            }
        }

        let selector = Selector::parse("li.productListItem").unwrap();
        let products = document.select(&selector);
        products
            .rev()
            .map(|product| Book {
                title: get_text(product, "h3 > a", false, "").unwrap(),
                author: get_text(product, "li.authorLabel > span", true, "Di:"),
                narrator: get_text(product, "li.narratorLabel > span", true, "Letto da:"),
                runtime: get_text(product, "li.runtimeLabel > span", false, "Durata:").unwrap(),
                date: get_text(
                    product,
                    "li.releaseDateLabel > span",
                    false,
                    "Data di pubblicazione:",
                )
                .unwrap(),
                url: Url::parse(
                    format!(
                        "https://example.com{}",
                        product
                            .select(&Selector::parse("a.bc-link").unwrap())
                            .next()
                            .map(|h3| h3.value().attr("href").unwrap().to_string())
                            .unwrap()
                    )
                    .as_str(),
                )
                .unwrap()
                .path()
                .to_string(),
            })
            .collect()
    }

    fn load_from_json(filename: &str) -> Vec<Self> {
        let json = std::fs::read_to_string(filename).unwrap();
        serde_json::from_str(&json).unwrap()
    }

    fn write_to_file(books: Vec<Self>, filename: &str) {
        let json = serde_json::to_string_pretty(&books).unwrap();
        std::fs::write(filename, json).unwrap();
    }

    fn formatted_message(&self, settings: &Settings) -> String {
        let attributes_map = &settings.attribute_names;
        let mut message = String::new();
        message.push_str(&format!("{}: {}\n", attributes_map.title, self.title));
        if let Some(author) = &self.author {
            message.push_str(&format!("{}: {}\n", attributes_map.author, author));
        }
        if let Some(narrator) = &self.narrator {
            message.push_str(&format!("{}: {}\n", attributes_map.narrator, narrator));
        }
        message.push_str(&format!("{}: {}\n", attributes_map.runtime, self.runtime));
        message.push_str(&format!("{}: {}\n", attributes_map.date, self.date));

        let url = format!("{}{}", settings.url_header, self.url);
        let url_message = format!("[{}]({})", settings.book_url_message, url);
        message.push_str(&url_message.as_str());
        message
    }

    fn formatted_log(&self) -> String {
        format!(
            "t: {} - a: {} - n: {} - r: {} - d: {} - u: {}",
            self.title,
            self.author.as_ref().unwrap_or(&"".to_string()),
            self.narrator.as_ref().unwrap_or(&"".to_string()),
            self.runtime,
            self.date,
            self.url
        )
    }
}

impl PartialEq for Book {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
            && self.author == other.author
            && self.narrator == other.narrator
            && self.date == other.date
            && self.runtime == other.runtime
    }
}

impl Eq for Book {}

fn update_log_file(str_to_append: &String) -> String {
    let timestamped_str = format!(
        "{} - {}\n",
        chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        str_to_append
    );
    let mut log_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(LOG_FILE)
        .unwrap();
    if let Err(e) = write!(log_file, "{}", timestamped_str) {
        eprintln!("Couldn't write to file: {}", e);
    }
    timestamped_str
}

fn books_update(settings: Settings) -> String {
    let mut log_str = "".to_string();
    let response = reqwest::blocking::get(&settings.url);
    let html_content = response.unwrap().text().unwrap();
    let document = Html::parse_document(&html_content);
    let books = Book::from_html_document(document);

    println!("Books to send: {:#?}", books.len());
    let mut stored_books = Book::load_from_json(BOOKS_FILE);
    let books_to_send = books
        .into_iter()
        .filter(|b| !stored_books.contains(b))
        .collect::<Vec<Book>>();

    // TODO: send on telegram
    for book in &books_to_send {
        log_str += &update_log_file(&book.formatted_log());
        let message = book.formatted_message(&settings);
    }

    stored_books.extend(books_to_send);

    let tot_books = stored_books.len();

    if tot_books > settings.max_books_kept as usize {
        stored_books.drain(0..(tot_books - settings.max_books_kept as usize));
    }
    println!("Books to write: {:#?}", stored_books.len());
    Book::write_to_file(stored_books, BOOKS_FILE);

    log_str
}

fn main() {
    // TODO: scheduling
    let mut log_str = update_log_file(&"Startup.".to_string());

    let bot = Bot::from_env(); // Needs TELOXIDE_TOKEN env variable

    let settings = Settings::from_file(SETTINGS_FILE);
    let bot_state = BotState::from_file(BOT_STATE_FILE);

    // TODO: telegram commands

    if bot_state.active {
        log_str += &update_log_file(&"Update begins.".to_string());
        books_update(settings);
        log_str += &update_log_file(&"Update ends.".to_string());
    }

    // TODO: send log
}
