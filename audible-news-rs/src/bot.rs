use crate::book;
use crate::settings;
use teloxide::{adaptors::DefaultParseMode, prelude::Requester, types::ChatId, Bot};
use tokio::runtime;

pub struct TelegramBot {
    pub bot: DefaultParseMode<Bot>,
    pub channel_id: ChatId,
    pub admin_chat_id: ChatId,
    pub max_message_length: u32,
}

impl TelegramBot {
    async fn send_message_async(
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

    pub fn send_message_blocking(
        &self,
        mut message: String,
        private: bool,
    ) -> Result<(), teloxide::RequestError> {
        let rt = runtime::Builder::new_multi_thread().enable_all().build()?;
        while message.len() > 0 {
            let to_send = message
                .drain(..std::cmp::min(message.len(), self.max_message_length as usize))
                .collect::<String>();
            rt.block_on(self.send_message_async(to_send, private))?;
        }
        Ok(())
    }

    pub fn send_book(
        &self,
        book: &book::Book,
        settings: &settings::Settings,
    ) -> teloxide::requests::ResponseResult<()> {
        let message = book.formatted_message(settings);
        println!("Sending message: {}", message);
        self.send_message_blocking(message, false)
    }
}
