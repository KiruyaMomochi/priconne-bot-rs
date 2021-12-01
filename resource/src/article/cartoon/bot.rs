use super::*;

use crate::{article::cartoon::Thumbnail, message::MessageBuilder, Bot};
use futures::StreamExt;
use priconne_core::Error;
use std::pin::Pin;
use teloxide::{
    payloads::SendPhotoSetters,
    prelude::Requester,
    types::{ChatId, Message},
};

struct CartoonMessageBuilder<'a> {
    pub thumbnail: &'a Thumbnail,
    pub page: &'a CartoonPage,
}

impl<'a> MessageBuilder for CartoonMessageBuilder<'a> {
    fn build_message(&self) -> String {
        let episode = &self.thumbnail.episode;
        let title = &self.thumbnail.title;
        let image_url = &self.page.image_src;
        let id = &self.thumbnail.id;

        let message = format!(
            "<b>第 {episode} 話</b>: {title}\n{image_url} <code>#{id}</code>",
            episode = episode,
            title = title,
            image_url = image_url,
            id = id
        );

        message
    }
}

impl<C: CartoonClient + Clone + Send> Bot<C> {
    pub async fn cartoon_by_id(&self, id: i32, chat_id: ChatId) -> Result<Message, Error> {
        let page = self.client.cartoon_detail(id).await?;
        let caption = format!("<code>#{}</code>\n{}", page.id, page.image_src);
        let message = self
            .bot
            .send_photo(chat_id, teloxide::types::InputFile::Url(page.image_src))
            .caption(caption)
            .parse_mode(teloxide::types::ParseMode::Html)
            .await?;

        Ok(message)
    }

    pub async fn cartoon(&self, thumbnail: &Thumbnail, chat_id: ChatId) -> Result<Message, Error> {
        let page = self.client.cartoon_detail(thumbnail.id).await?;
        let message = CartoonMessageBuilder {
            page: &page,
            thumbnail: &thumbnail,
        };
        let message = message.build_message();
        let message = self
            .bot
            .send_photo(chat_id, teloxide::types::InputFile::Url(page.image_src))
            .caption(message)
            .parse_mode(teloxide::types::ParseMode::Html)
            .send()
            .await?;

        Ok(message)
    }

    pub async fn cartoon_all(&self, limit: i32, min: i32, chat_id: ChatId) -> Result<(), Error> {
        log::info!("cartoon_all with limit {} and min {}", limit, min);

        let stream = self.client.cartoon_stream();
        let mut stream = unsafe { Pin::new_unchecked(stream) };

        let mut skip_counter = 0;
        let mut vec = Vec::new();
        while let Some(cartoon) = stream.next().await {
            if skip_counter >= limit || cartoon.id < min {
                break;
            }

            let sent_cartoon = self.mongo_database.check_cartoon(&cartoon).await?;
            
            if sent_cartoon.is_none() {
                log::info!("hit cartoon {}: {}", cartoon.episode, cartoon.title);
                if cartoon.id >= min {
                    skip_counter = 0;
                }

                vec.push(cartoon);
            } else {
                skip_counter += 1;
                log::info!(
                    "ign cartoon {}: {} ({}/{})",
                    cartoon.episode,
                    cartoon.title,
                    skip_counter,
                    limit
                );
            }
        }

        for cartoon in vec.iter().rev() {
            self.cartoon(cartoon, chat_id.clone()).await?;
            self.mongo_database.upsert_cartoon(cartoon).await?;
        }

        Ok(())
    }
}
