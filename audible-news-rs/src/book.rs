use crate::settings;
use crate::utils;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use teloxide::utils::markdown;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Book {
    title: String,
    author: Option<String>,
    narrator: Option<String>,
    runtime: String,
    date: String,
    url: String,
}

impl Book {
    fn from_html_node(node: ElementRef) -> Result<Book, Box<dyn std::error::Error>> {
        fn get_text<'a>(
            node: ElementRef,
            selector: &'a str,
            remove: &str,
        ) -> Result<String, Box<dyn std::error::Error + 'a>> {
            Ok(node
                .select(&Selector::parse(selector)?)
                .next()
                .map(|p| p.text().collect::<String>())
                .ok_or(utils::TBotError::BookFieldNotFound(String::from(selector)))?
                .replace(remove, "")
                .trim()
                .to_string())
        }
        let title = get_text(node, "h3 > a", "")?;
        let book_url = node
            .select(&Selector::parse("a.bc-link")?)
            .next()
            .map(|h3| h3.value().attr("href"))
            .ok_or(utils::TBotError::BookFieldNotFound(String::from("url")))?
            .ok_or(utils::TBotError::BookFieldNotFound(String::from("url")))?;
        Ok(Book {
            title: title,
            author: match get_text(node, "li.authorLabel > span", "Di:") {
                Ok(s) => Some(s),
                Err(_) => None,
            },
            narrator: match get_text(node, "li.narratorLabel > span", "Letto da:") {
                Ok(s) => Some(s),
                Err(_) => None,
            },
            runtime: get_text(node, "li.runtimeLabel > span", "Durata:")?,
            date: get_text(node, "li.releaseDateLabel > span", "Data di pubblicazione:")?,
            url: url::Url::parse(format!("https://example.com{}", book_url).as_str())?
                .path()
                .to_string(),
        })
    }

    pub fn from_html_document(document: Html) -> Result<Vec<Book>, Box<dyn std::error::Error>> {
        let selector = Selector::parse("li.productListItem")?;
        let products = document.select(&selector);
        let mut books = Vec::new();
        for product in products.rev() {
            let book = Book::from_html_node(product)?;
            books.push(book);
        }
        Ok(books)
    }

    pub fn load_from_json(filename: &str) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(filename)?;
        let books: Vec<Self> = serde_json::from_str(&json)?;
        Ok(books)
    }

    pub fn store_to_json(
        mut books: Vec<Self>,
        filename: &str,
        max_books: u32,
    ) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        let tot_books = books.len();
        if tot_books > max_books as usize {
            books.drain(0..(tot_books - max_books as usize));
        }
        let json = serde_json::to_string_pretty(&books)?;
        std::fs::write(filename, json)?;
        return Ok(books);
    }

    pub fn formatted_message(&self, settings: &settings::Settings) -> String {
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

        message = utils::md_escape(&message); // Escape dots

        let complete_url = format!("{}{}", settings.url_header, self.url);
        let url_message = markdown::link(complete_url.as_str(), settings.book_url_message.as_str());
        message.push_str(&url_message.as_str());
        message
    }

    pub fn formatted_log(&self) -> String {
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
