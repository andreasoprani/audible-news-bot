use crate::settings;
use crate::utils;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use teloxide::utils::markdown;

#[derive(Debug, Serialize, Deserialize, Clone, Eq)]
pub struct Book {
    title: String,
    author: Option<String>,
    narrator: Option<String>,
    runtime: String,
    date: String,
    url: String,
}

impl PartialEq for Book {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
            && self.author == other.author
            && self.narrator == other.narrator
            && self.runtime == other.runtime
            && self.date == other.date
    }
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
        let selector = Selector::parse("li.productListItem[id]")?;
        let products = document.select(&selector);
        let mut books = Vec::new();
        for product in products.rev() {
            let book = Book::from_html_node(product)?;
            books.push(book);
        }
        Ok(books)
    }

    pub fn has_ai_narrator(&self) -> bool {
        self.narrator.as_ref().is_some_and(|narrator| {
            let narrator = narrator.to_lowercase();
            narrator.contains("virtual voice") || narrator.contains("ai voice")
        })
    }

    pub fn limit(mut books: Vec<Self>, max_books: u32) -> Vec<Self> {
        let tot_books = books.len();
        if tot_books > max_books as usize {
            books.drain(0..(tot_books - max_books as usize));
        }
        books
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

#[cfg(test)]
mod tests {
    use super::Book;
    use scraper::Html;

    fn sample_book() -> Book {
        Book {
            title: "Test book".into(),
            author: Some("Jane Smith".into()),
            narrator: None,
            runtime: "1 hour".into(),
            date: "2026-01-01".into(),
            url: "/test-book".into(),
        }
    }

    #[test]
    fn parses_catalogue_fields_and_optional_narrator() {
        let html = Html::parse_document(
            r#"<li class="productListItem" id="book">
                <h3><a class="bc-link" href="/pd/test-book">Test book</a></h3>
                <ul>
                    <li class="authorLabel"><span>Di: Jane Smith</span></li>
                    <li class="runtimeLabel"><span>Durata: 1 hour</span></li>
                    <li class="releaseDateLabel"><span>Data di pubblicazione: 2026-01-01</span></li>
                </ul>
            </li>"#,
        );
        let books = Book::from_html_document(html).unwrap();
        assert_eq!(books.len(), 1);
        assert_eq!(books[0], sample_book());
        assert_eq!(books[0].url, "/pd/test-book");
    }

    #[test]
    fn parsing_empty_catalogue_returns_no_books() {
        let books = Book::from_html_document(Html::parse_document("<html></html>")).unwrap();
        assert!(books.is_empty());
    }

    #[test]
    fn parsing_book_without_required_fields_fails() {
        let html = Html::parse_document(r#"<li class="productListItem" id="book"></li>"#);
        assert!(Book::from_html_document(html).is_err());
    }

    #[test]
    fn limit_keeps_newest_books_and_handles_boundaries() {
        let books: Vec<Book> = ["Oldest", "Middle", "Newest"]
            .into_iter()
            .map(|title| Book {
                title: title.into(),
                ..sample_book()
            })
            .collect();

        assert_eq!(Book::limit(books.clone(), 2), books[1..]);
        assert_eq!(Book::limit(books.clone(), 3), books);
        assert_eq!(Book::limit(books.clone(), 4), books);
        assert!(Book::limit(books, 0).is_empty());
        assert!(Book::limit(Vec::new(), 2).is_empty());
    }

    #[test]
    fn book_json_round_trip_preserves_fields() {
        let book = sample_book();
        let json = serde_json::to_string(&book).unwrap();
        let restored: Book = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, book);
        // URLs are deliberately not part of Book equality.
        assert_eq!(restored.url, book.url);
    }

    #[test]
    fn detects_ai_narrators() {
        for (narrator, expected) in [
            (Some("Virtual Voice"), true),
            (Some("AI Voice"), true),
            (Some(" virtual voice "), true),
            (Some("ai VOICE"), true),
            (Some("Jane Smith, Virtual Voice"), true),
            (Some("AI Voice, John Smith"), true),
            (Some("Jane Smith"), false),
            (Some(""), false),
            (None, false),
        ] {
            let book = Book {
                title: "Test book".into(),
                author: None,
                narrator: narrator.map(str::to_string),
                runtime: "1 hour".into(),
                date: "2026-01-01".into(),
                url: "/test-book".into(),
            };
            assert_eq!(book.has_ai_narrator(), expected, "{narrator:?}");
        }
    }
}
