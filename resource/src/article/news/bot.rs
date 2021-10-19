use super::*;

use crate::{bot::Bot, message::MessageBuilder};
use futures::StreamExt;
use linked_hash_set::LinkedHashSet;
use priconne_core::{Error, Tagger};
use std::pin::Pin;
use telegraph_rs::doms_to_nodes;
use teloxide::types::{ChatId, Message};
use utils::SplitPrefix;

pub struct NewsMessageBuilder<'a> {
    pub page: &'a NewsPageNoContent,
    pub news_id: i32,
    pub telegraph_page: &'a telegraph_rs::Page,
    pub tagger: &'a Tagger,
}

impl<'a> MessageBuilder for NewsMessageBuilder<'a> {
    fn build_message(&self) -> String {
        let (title, tags) = tags(self.page, self.tagger);

        let link = &self.telegraph_page.url;
        let id = self.news_id;
        let time = self.page.date.format("%Y-%m-%d").to_string();

        let mut tag_str = String::new();

        for tag in &tags {
            tag_str.push_str("#");
            tag_str.push_str(tag);
            tag_str.push_str(" ");
        }

        if !tag_str.is_empty() {
            tag_str.pop();
            tag_str.push('\n');
        }

        let message = format!(
            "{tag}<b>{title}</b>\n{link}\n{time} <code>News#{id}</code>",
            tag = tag_str,
            title = title,
            link = link,
            time = time,
            id = id
        );

        message
    }
}

pub struct SentNews {
    message: Message,
    telegraph: telegraph_rs::Page,
    page: NewsPageNoContent,
}

impl<C: NewsClient + Clone + Send> Bot<C> {
    pub async fn news_by_id(&self, news_id: i32, chat_id: ChatId) -> Result<SentNews, Error> {
        let href = self.client.news_detail_href(news_id);
        return self.news_href(&href, news_id, chat_id).await;
    }

    pub async fn news(&self, news: &News, chat_id: ChatId) -> Result<SentNews, Error> {
        return self.news_href(&news.href, news.id, chat_id).await;
    }

    async fn news_href(&self, href: &str, id: i32, chat_id: ChatId) -> Result<SentNews, Error> {
        let page;
        let content;
        {
            let (p, c) = self.client.news_page_from_href(href).await?.split();
            page = p;
            content = serde_json::to_string(&doms_to_nodes(c.children()))?;
        };

        let telegraph_page = self
            .telegraph
            .create_page(&page.title, &content, false)
            .await?;

        let message_builder = NewsMessageBuilder {
            news_id: id,
            page: &page,
            telegraph_page: &telegraph_page,
            tagger: &self.tagger,
        };

        let disable_notification = page.title.contains("外掛停權");

        let message = self
            .bot
            .send_message(chat_id, message_builder.build_message())
            .parse_mode(ParseMode::Html)
            .disable_notification(disable_notification)
            .await?;

        Ok(SentNews {
            message,
            page,
            telegraph: telegraph_page,
        })
    }

    pub async fn news_all(&self, limit: i32, min: i32, chat_id: ChatId) -> Result<(), Error> {
        log::info!("news_all with limit {} and min {}", limit, min);

        let stream = self.client.news_stream();
        let mut stream = unsafe { Pin::new_unchecked(stream) };

        let mut skip_counter = 0;
        let mut vec = Vec::new();
        while let Some(news) = stream.next().await {
            if skip_counter >= limit {
                break;
            }

            let sent_news = self.mongo_database.check_sent_news(&news).await?;
            if sent_news.is_not_found() {
                log::info!("hit news {}: {}", news.id, news.display_title);
                if news.id >= min {
                    skip_counter = 0;
                }

                vec.push((news, sent_news));
            } else {
                skip_counter += 1;
                log::info!(
                    "ign news {}: {} ({}/{})",
                    news.id,
                    news.display_title,
                    skip_counter,
                    limit
                );
            }
        }

        for (news, result) in vec.iter().rev() {
            let message = self.news(&news, chat_id.clone()).await?;
            self.mongo_database
                .upsert_sent_news(&result, news, &message.message, &message.telegraph.url)
                .await?;
        }

        Ok(())
    }
}

fn tags<'a>(page: &'a NewsPageNoContent, tagger: &'a Tagger) -> (&'a str, Vec<String>) {
    let mut title: &str = &page.title;
    let mut tags: LinkedHashSet<String> = LinkedHashSet::new();

    if let Some(category) = &page.category {
        tags.insert(category.to_owned());
    }
    if let Some((category, base_title)) = title.split_prefix('【', '】') {
        title = base_title;
        tags.insert(category.to_owned());
    }

    tags.extend(tagger.tag(title));
    (title, tags.into_iter().collect())
}
