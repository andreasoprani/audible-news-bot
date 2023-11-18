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
