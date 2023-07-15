use mongodb::Collection;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use teloxide::{
    payloads::{SendMessageSetters, SendPhotoSetters},
    requests::{Request, Requester},
    types::{ChatId, InputFile, MessageId, Recipient},
};

use crate::{resource::Announcement, Error};

pub struct ChatManager {
    pub bot: teloxide::Bot,
    pub post_recipient: Recipient,
    pub cartoon_recipient: Recipient,
    pub messages: Collection<SendResult>,
}

/// Message send result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendResult {
    pub url: Option<Url>,
    pub recipient: Recipient,
    /// Chat id of the recipient, may be the same as `recipient`,
    /// but we store it for a more stable reference
    pub chat_id: ChatId,
    pub message_id: MessageId,
    pub resource_id: crate::resource::ResourceId,
    pub update_time: chrono::DateTime<chrono::Utc>,
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

    pub async fn send_announcement(
        &self,
        post: &Announcement,
    ) -> Result<teloxide::prelude::Message, Error> {
        let message = self
            .send_to(post.message(), self.post_recipient.clone())
            .await?;
        self.messages
            .insert_one(
                SendResult {
                    recipient: self.post_recipient.clone(),
                    chat_id: message.chat.id,
                    resource_id: crate::resource::ResourceId::Announcement(post.id),
                    update_time: message.date,
                    message_id: message.id,
                    url: message.url(),
                },
                None,
            )
            .await?;
        Ok(message)
    }

    pub async fn send_cartoon<M: Sendable>(
        &self,
        cartoon: &M,
    ) -> Result<teloxide::prelude::Message, Error> {
        self.send_to(cartoon.message(), self.cartoon_recipient.clone())
            .await
    }
}
