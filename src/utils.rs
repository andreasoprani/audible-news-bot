use std::fmt;

pub fn md_escape(s: &String) -> String {
    let mut s = s.clone();
    for c in [
        '\\', '*', '_', '`', '{', '}', '[', ']', '(', ')', '#', '+', '-', '.', '!', '|',
    ] {
        s = s.replace(c, &format!("\\{}", c));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::md_escape;

    #[test]
    fn escapes_markdown_special_characters() {
        assert_eq!(
            md_escape(&r"A *title* [link](url). C:\books".to_string()),
            r"A \*title\* \[link\]\(url\)\. C:\\books"
        );
    }

    #[test]
    fn leaves_plain_text_unchanged() {
        assert_eq!(
            md_escape(&"An ordinary title".to_string()),
            "An ordinary title"
        );
        assert_eq!(md_escape(&String::new()), "");
    }
}

#[derive(Debug)]
pub enum TBotError {
    BookFieldNotFound(String),
}

impl fmt::Display for TBotError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TBotError::BookFieldNotFound(ref s) => write!(f, "Book field not found: {}", s),
        }
    }
}

impl std::error::Error for TBotError {
    fn description(&self) -> &str {
        match *self {
            TBotError::BookFieldNotFound(ref s) => s,
        }
    }
}
