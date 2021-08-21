use futures::StreamExt;
use std::pin::Pin;
use teloxide::{
    adaptors::AutoSend,
    payloads::{SendMessageSetters, SendPhotoSetters},
    prelude::Requester,
    types::{ChatId, InputFile, Message, ParseMode},
};

use crate::bot::Bot;
use crate::{article::cartoon::Thumbnail, error::Error, message::MessageBuilder};

use super::{CartoonBot, CartoonClient, CartoonDatabase, CartoonExt, CartoonMessageBuilder};

impl<C: CartoonClient + Clone> Bot<C> {
    pub async fn cartoon_upsert(&self, thumbnail: &Thumbnail) -> Result<Option<Message>, Error> {
        if self
            .mongo_database
            .check_cartoon(thumbnail)
            .await?
            .is_some()
        {
            return Ok(None);
        }

        let page = self.client.cartoon_detail(thumbnail.id).await?;
        let message = CartoonMessageBuilder {
            page: &page,
            thumbnail: &thumbnail,
        };
        let _chat_id = ChatId::ChannelUsername("@pcrtwstat".to_owned());
        let message = self
            .bot
            .send_cartoon(
                self.chat_id.clone(),
                page.image_src.to_owned(),
                message.build_message(),
            )
            .await?;

        self.mongo_database.upsert_cartoon(thumbnail).await?;

        Ok(Some(message))
    }

    pub async fn cartoon_all(&self) -> Result<(), Error> {
        let stream = self.client.cartoon_stream();
        let mut stream = unsafe { Pin::new_unchecked(stream) };

        let mut skip_counter = 0;
        while let Some(cartoon) = stream.next().await {
            if let Some(_message) = self.cartoon_upsert(&cartoon).await? {
                skip_counter = 0;
            } else {
                skip_counter += 1;
            }

            if skip_counter == 5 {
                break;
            }
        }

        Ok(())
    }
}
