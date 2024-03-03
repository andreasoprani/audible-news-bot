use crate::book;
use crate::settings;
use teloxide::{
    adaptors::DefaultParseMode,
    prelude::Requester,
    requests::RequesterExt,
    types::{ChatId, ParseMode},
    Bot,
};

pub struct TelegramBot {
    pub bot: DefaultParseMode<Bot>,
    pub channel_id: ChatId,
    pub admin_chat_id: ChatId,
    pub max_message_length: u32,
}

impl TelegramBot {
    pub fn new(settings: &settings::Settings) -> Result<Self, Box<dyn std::error::Error>> {
        let bot = TelegramBot {
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
        Ok(bot)
    }

    pub async fn send_message_async(
        &self,
        message: String,
        private: bool,
    ) -> teloxide::requests::ResponseResult<()> {
        loop {
            match (match private {
                false => self.bot.send_message(self.channel_id, message.clone()),
                true => self
                    .bot
                    .inner()
                    .send_message(self.admin_chat_id, message.clone()),
            })
            .await
            {
                Ok(_) => return Ok(()),
                Err(teloxide::RequestError::RetryAfter(t)) => {
                    println!("Retry after: {:?}", t);
                    tokio::time::sleep(t).await;
                }
                Err(e) => {
                    eprintln!("Error sending message: {}", e);
                    return Err(e);
                }
            }
        }
    }

    pub async fn send_book(
        &self,
        book: &book::Book,
        settings: &settings::Settings,
    ) -> teloxide::requests::ResponseResult<()> {
        let message = book.formatted_message(settings);
        println!("Sending message: {}", message);
        self.send_message_async(message, false).await
    }
}
