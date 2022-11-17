use crate::bot::Bot;
use futures::StreamExt;
use linked_hash_set::LinkedHashSet;
use priconne_core::{Error, RegexTagger};
use resource::{
    message::MessageBuilder,
    news::{News, NewsClient, NewsExt, NewsPage}
};
use std::pin::Pin;
use telegraph_rs::doms_to_nodes;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{ChatId, Message, ParseMode},
};
use crate::utils::{replace_relative_path, SplitPrefix};

pub struct NewsMessageBuilder<'a> {
    pub page: &'a NewsPage,
    pub news_id: i32,
    pub telegraph_page: &'a telegraph_rs::Page,
    pub tagger: &'a RegexTagger,
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

        let mut event_str = String::new();

        for event in &self.page.events {
            event_str.push_str("- ");
            event_str.push_str(&event.title);
            event_str.push_str(": \n   ");
            event_str.push_str(event.start.format("%m/%d %H:%M").to_string().as_str());
            event_str.push_str(" - ");
            event_str.push_str(event.end.format("%m/%d %H:%M").to_string().as_str());
            event_str.push_str("\n");
        }
        if !event_str.is_empty() {
            event_str.insert_str(0, "\n");
            event_str.push_str("\n");
        }

        let head = format!("{tag}<b>{title}</b>\n", tag = tag_str, title = title);

        let tail = format!(
            "{link}\n{time} <code>News#{id}</code>",
            link = link,
            time = time,
            id = id
        );

        let message = format!("{}{}{}", head, event_str, tail);
        message
    }
}

pub struct SentNews {
    message: Message,
    telegraph: telegraph_rs::Page,
    page: NewsPage,
}

#[derive(Debug)]
pub enum NewsSource<'a> {
    Id(i32),
    Href(&'a str, i32),
    News(&'a News),
}

impl<C: NewsClient + Clone + Send> Bot<C> {
    pub async fn send_news(
        &self,
        news: NewsSource<'_>,
        chat_id: ChatId,
    ) -> Result<SentNews, Error> {
        let mut string_owner = String::new();

        let (href, id) = match news {
            NewsSource::Href(href, id) => (href, id),
            NewsSource::News(news) => (news.href.as_str(), news.id),
            NewsSource::Id(id) => {
                string_owner = self.client.news_detail_href(id);
                (string_owner.as_str(), id)
            }
        };

        let ref this = self;
        let page;
        let content;
        {
            let (p, c) = this.client.news_page_from_href(href).await?;
            let url = this.client.news_url(href)?;
            let nodes = &mut doms_to_nodes(c.children());
            if let Some(nodes) = nodes {
                replace_relative_path(&url, nodes)?;
            };
            page = p;
            content = serde_json::to_string(&nodes)?;
        };

        let telegraph_page = this
            .telegraph
            .create_page(&page.title, &content, false)
            .await?;

        let message = NewsMessageBuilder {
            news_id: id,
            page: &page,
            telegraph_page: &telegraph_page,
            tagger: &this.tagger,
        }.build_message();

        let disable_notification = page.title.contains("外掛停權");

        let message = this
            .bot
            .send_message(chat_id, message)
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

            let sent_news = self.database.check_news_in_sent(&news).await?;
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
            let message = self.send_news(NewsSource::News(news), chat_id.clone()).await?;
            self.database.upsert_news(news).await?;
            self.database
                .upsert_news_in_sent(&result, news, &message.message, &message.telegraph.url)
                .await?;
        }

        Ok(())
    }
}

fn tags<'a>(page: &'a NewsPage, tagger: &'a RegexTagger) -> (&'a str, Vec<String>) {
    let mut title: &str = &page.title;
    let mut tags: LinkedHashSet<String> = LinkedHashSet::new();

    if let Some(category) = &page.category {
        tags.insert(category.to_owned());
    }
    if let Some((category, base_title)) = title.split_prefix('【', '】') {
        title = base_title;
        tags.insert(category.to_owned());
    }

    tags.extend(tagger.tag_iter(title));
    (title, tags.into_iter().collect())
}
