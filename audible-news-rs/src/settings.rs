use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AttributesMap {
    pub title: String,
    pub author: String,
    pub narrator: String,
    pub runtime: String,
    pub date: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub url: String,
    pub url_header: String,
    pub max_books_kept: u32,
    pub max_message_length: u32,
    pub default_log_length: u32,
    pub allowed_commands: Vec<String>,
    pub attribute_names: AttributesMap,
    pub redirect_message: String,
    pub book_url_message: String,
}

impl Settings {
    pub fn from_file(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(filename)?;
        match serde_json::from_str(&json) {
            Ok(s) => Ok(s),
            Err(e) => Err(Box::new(e)),
        }
    }
}
