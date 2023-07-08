use reqwest::Url;
use teloxide::{
    payloads::{SendMessageSetters, SendPhotoSetters},
    requests::{Request, Requester},
    types::{InputFile, Recipient},
};

use crate::Error;

pub struct ChatManager {
    pub bot: teloxide::Bot,
    pub post_recipient: Recipient,
    pub cartoon_recipient: Recipient,
}

pub struct SendResult {
    pub url: String,
    pub recipient: Recipient,
}

pub struct Message {
    pub text: String,
    pub silent: bool,
    pub image_src: Option<Url>,
}

pub trait Sendable {
    fn message(&self) -> Message;
}

impl ChatManager {
    async fn send_to<C>(
        &self,
        message: Message,
        chat_id: C,
    ) -> Result<teloxide::prelude::Message, Error>
    where
        C: Into<Recipient> + Clone,
    {
        let send_result = if let Some(image_src) = message.image_src {
            self.bot
                .send_photo(chat_id.clone(), InputFile::url(image_src))
                .caption(message.text)
                .disable_notification(message.silent)
                .parse_mode(teloxide::types::ParseMode::Html)
                .send()
                .await?
        } else {
            self.bot
                .send_message(chat_id.clone(), message.text.clone())
                .disable_notification(message.silent)
                .parse_mode(teloxide::types::ParseMode::Html)
                .send()
                .await?
        };

        Ok(send_result)
    }

    pub async fn send_post<M: Sendable>(
        &self,
        post: &M,
    ) -> Result<teloxide::prelude::Message, Error> {
        self.send_to(post.message(), self.post_recipient.clone())
            .await
    }

    pub async fn send_cartoon<M: Sendable>(
        &self,
        cartoon: &M,
    ) -> Result<teloxide::prelude::Message, Error> {
        self.send_to(cartoon.message(), self.cartoon_recipient.clone())
            .await
    }
}
