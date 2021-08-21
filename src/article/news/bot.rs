use futures::StreamExt;
use log::info;
use std::{pin::Pin, time::Duration};
use teloxide::{
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{ChatId, Message, ParseMode},
};

use crate::{
    bot::Bot,
    database::{PriconneNewsDatabase, SentMessage},
    error::Error,
    message::{map_titie, MessageBuilder, Tagger},
    page::Page,
    utils::SplitPrefix,
};

use super::{News, NewsBot, NewsClient, NewsDatabase, NewsExt, NewsMessageBuilder};

impl<C: NewsClient + Clone> Bot<C> {
    pub async fn news(&self, news: &News) -> Result<Option<Message>, Error> {
        let sent_news = self.mongo_database.check_sent_news(news).await?;

        if sent_news.is_found() {
            return Ok(None);
        }

        let page = self.client.news_page_from_href(&news.href).await?;

        // let nodes = telegraph_rs::doms_to_nodes(page.content.children());
        let telegraph_page = self
            .telegraph
            .create_page_doms(&page.title, page.content.children(), false)
            .await?;

        let message_builder = NewsMessageBuilder {
            news: &news,
            page: &page,
            telegraph_page: &telegraph_page,
            tagger: &self.tagger,
        };

        let message = self
            .bot
            .send_news(self.chat_id.clone(), message_builder.build_message())
            .await?;

        self.mongo_database
            .upsert_sent_news(
                &sent_news,
                &news,
                &message,
                &message_builder.telegraph_page.url,
            )
            .await?;

        Ok(Some(message))
    }

    pub async fn news_all(&self) -> Result<(), Error> {
        let stream = self.client.news_stream();
        let mut stream = unsafe { Pin::new_unchecked(stream) };

        let mut skip_counter = 0;
        while let Some(news) = stream.next().await {
            tokio::time::sleep(Duration::from_secs(1)).await;
            info!("news {}", news.title);

            if let Some(_message) = self.news(&news).await? {
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
