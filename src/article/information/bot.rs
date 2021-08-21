use futures::StreamExt;
use log::info;
use std::pin::Pin;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{Message, ParseMode},
};

use crate::{
    article::information::InformationDatabase,
    bot::Bot,
    database::{PriconneNewsDatabase, SentMessage},
    error::Error,
    message::{map_titie, MessageBuilder, Tagger},
    utils::SplitPrefix,
};

use super::{Announce, InformationClient, InformationExt, InformationMessageBuilder};

impl<C: InformationClient + Clone> Bot<C> {
    pub async fn announce(&self, announce: &Announce) -> Result<Option<Message>, Error> {
        let sent_information = self.mongo_database.check_sent_news(announce).await?;
        if sent_information.is_found() {
            return Ok(None);
        }

        let page = self.client.information_page(announce.announce_id).await?;
        info!("Got information page {}", page.title);

        let telegraph_page = self
            .telegraph
            .create_page_doms(&page.title, page.content.children(), false)
            .await?;
        info!("Published telegraph page {}", telegraph_page.url);

        let message_builder = InformationMessageBuilder {
            announce: announce,
            telegraph_page: &telegraph_page,
            page: &page,
            tagger: &self.tagger,
        };

        let message = self
            .bot
            .send_message(self.chat_id.clone(), message_builder.build_message())
            .parse_mode(ParseMode::Html)
            .disable_notification(false)
            .await?;

        info!("Message sent");

        self.mongo_database.upsert_announce(announce).await?;
        self.mongo_database
            .upsert_sent_information(
                &sent_information,
                &announce,
                &message,
                &message_builder.telegraph_page.url,
            )
            .await?;

        Ok(Some(message))
    }

    pub async fn announce_all(&self) -> Result<(), Error> {
        let stream = self.client.information_stream();
        let mut stream = unsafe { Pin::new_unchecked(stream) };

        let mut skip_counter = 0;
        while let Some(announce) = stream.next().await {
            info!("announce {}", announce.title.title);

            if let Some(_message) = self.announce(&announce).await? {
                skip_counter = 0;
            } else {
                skip_counter += 1;
            }

            if skip_counter == 10 {
                break;
            }
        }

        Ok(())
    }
}
