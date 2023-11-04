use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

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
                url: product
                    .select(&Selector::parse("h3 > a").unwrap())
                    .next()
                    .map(|h3| h3.value().attr("href").unwrap().to_string())
                    .unwrap(),
            })
            .collect()
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
}

fn main() {
    let settings = Settings::from_file("data/bot_settings.json");
    let response = reqwest::blocking::get(&settings.url);
    let html_content = response.unwrap().text().unwrap();
    let document = Html::parse_document(&html_content);
    let books = Book::from_html_document(document);
    println!("{:#?}", books.len());
    println!("{}", books.first().unwrap().formatted_message(&settings));

    // TODO: remove books already sent (in json)
    // TODO: write to json and cut json file
    // TODO: telegram sending
    // TODO: telegram commands
    // TODO: log
    // TODO: scheduling
}
