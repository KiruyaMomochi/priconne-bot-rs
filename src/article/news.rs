mod bot;
mod page;

use async_trait::async_trait;
use linked_hash_set::LinkedHashSet;
use mongodb::{
    bson::{doc, Document},
    options::{FindOneAndReplaceOptions, FindOneOptions},
};
use reqwest::Response;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::{Request, Requester},
    types::{Message, ParseMode},
};

use crate::{
    client::Client,
    database::{PriconneNewsDatabase, SentMessage},
    error::Error,
    message::{map_titie, MessageBuilder, Tagger},
    utils::SplitPrefix,
};
pub use page::*;

use futures::StreamExt;
use kuchiki::traits::TendrilSink;

use crate::page::Page;

impl<T: ?Sized> NewsExt for T where T: NewsClient + Clone {}

pub enum NewsResult {
    News(News),
    SentNews(SentMessage),
    SentNoNews(SentMessage),
    None,
}

impl NewsResult {
    pub fn is_found(&self) -> bool {
        match self {
            NewsResult::News(_) => true,
            NewsResult::SentNews(_) => true,
            NewsResult::SentNoNews(_) => false,
            NewsResult::None => false,
        }
    }

    pub fn is_not_found(&self) -> bool {
        return !self.is_found();
    }
}

#[async_trait]
trait NewsDatabase: PriconneNewsDatabase {
    fn news(&self) -> mongodb::Collection<News>;

    async fn find_news(&self, news: &News) -> Result<Option<News>, mongodb::error::Error> {
        let collection = self.news();
        let filter = news_filter(news);
        collection.find_one(filter, None).await
    }

    async fn check_news(&self, news: &News) -> Result<Option<News>, mongodb::error::Error> {
        if let Some(found_news) = self.find_news(news).await? {
            if found_news.display_title == news.title && found_news.date == news.date {
                return Ok(Some(found_news));
            }
        }

        Ok(None)
    }

    async fn check_sent_news(&self, news: &News) -> Result<NewsResult, mongodb::error::Error> {
        if let Some(news) = self.check_news(news).await? {
            return Ok(NewsResult::News(news));
        }

        let found_sent = self.find_sent_news(news).await?;
        if let Some(found_sent) = found_sent {
            if found_sent.news_id.is_none() {
                self.upsert_news(news);
                self.update_sent_news(&found_sent, news).await?;
                return Ok(NewsResult::SentNews(found_sent));
            }
            return Ok(NewsResult::SentNoNews(found_sent));
        }

        Ok(NewsResult::None)
    }

    async fn upsert_news(&self, news: &News) -> Result<Option<News>, mongodb::error::Error> {
        let collection = self.news();
        let filter = news_filter(news);
        let options = Some(FindOneAndReplaceOptions::builder().upsert(true).build());
        let replace_result = collection
            .find_one_and_replace(filter, news, options)
            .await?;
        Ok(replace_result)
    }

    async fn find_sent_news(
        &self,
        news: &News,
    ) -> Result<Option<SentMessage>, mongodb::error::Error> {
        let collection = self.sent_messages();
        let filter = sent_news_filter(news);
        let find_result = collection
            .find_one(
                filter,
                FindOneOptions::builder()
                    .sort(doc! {"update_time": -1})
                    .build(),
            )
            .await;

        find_result
    }

    async fn update_sent_news(
        &self,
        sent: &SentMessage,
        news: &News,
    ) -> Result<Option<SentMessage>, mongodb::error::Error> {
        let sent_collection = self.sent_messages();
        let find_result = sent_collection
            .find_one_and_update(
                doc! {
                    "message_id": sent.message_id
                },
                doc! {
                    "$currentDate": {
                        "update_time": true,
                    },
                    "$set": {
                        "news_id": news.id,
                    },
                },
                None,
            )
            .await;

        find_result
    }

    async fn upsert_sent_news(
        &self,
        found_sent: &NewsResult,
        news: &News,
        message: &Message,
        telegraph_url: &str,
    ) -> Result<(), mongodb::error::Error> {
        self.upsert_news(news);

        let sent_collection = self.sent_messages();
        match found_sent {
            NewsResult::None => {
                let sent_message = SentMessage {
                    announce_id: None,
                    mapped_title: map_titie(&news.title),
                    message_id: message.id,
                    news_id: Some(news.id),
                    telegraph_url: telegraph_url.to_owned(),
                    update_time: chrono::Utc::now(),
                };
                sent_collection.insert_one(sent_message, None).await?;
            }
            NewsResult::SentNoNews(found_sent) => {
                sent_collection
                    .find_one_and_update(
                        doc! {
                            "message_id": found_sent.message_id
                        },
                        doc! {
                            "$currentDate": {
                                "update_time": true,
                            },
                            "$set": {
                                "message_id": message.id,
                                "news_id": news.id,
                                "telegraph_url": telegraph_url.to_owned(),
                            },
                        },
                        None,
                    )
                    .await?;
            }
            _ => unreachable!(),
        };

        Ok(())
    }
}

fn sent_news_filter(news: &News) -> Document {
    let time = chrono::Utc::now() - chrono::Duration::hours(24);
    let mapped_title = &map_titie(&news.title);
    doc! {
        "$or": [
            {
                "mapped_title": mapped_title,
                "news_id": null,
                "update_time": {
                    "$gte": time
                }
            },
            {
                "mapped_title": mapped_title,
                "news_id": news.id,
            }
        ]
    }
}

fn news_filter(news: &News) -> Document {
    doc! {
        "id": news.id,
    }
}

impl NewsDatabase for mongodb::Database {
    fn news(&self) -> mongodb::Collection<News> {
        self.collection("news")
    }
}

pub trait NewsExt: NewsClient + Clone {
    fn news_stream(&self) -> Box<dyn futures::Stream<Item = News> + '_> {
        let stream =
            futures::stream::unfold((Some(self.news_list_href(1)), self.clone()), next_news_list);

        let stream =
            stream.flat_map(|news_list| futures::stream::iter(news_list.news_list.into_iter()));

        Box::new(stream)
    }
}

async fn next_news_list<T: NewsExt>(
    (href, client): (Option<String>, T),
) -> Option<(NewsList, (Option<String>, T))> {
    let href = href?;
    let response = client.news_get(&href).await.ok()?;
    let text = response.text().await.ok()?;
    let document = kuchiki::parse_html().one(text);
    let news_list = NewsList::from_document(document).ok()?;
    let next_href = news_list.next_href.clone();

    Some((news_list, (next_href, client)))
}

#[async_trait]
trait NewsBot {
    async fn send_news<'a, C>(
        &self,
        chat_id: C,
        message: String,
    ) -> Result<teloxide::types::Message, teloxide::RequestError>
    where
        C: Into<teloxide::types::ChatId> + Send;
}

#[async_trait]
impl NewsBot for teloxide::Bot {
    async fn send_news<'a, C>(
        &self,
        chat_id: C,
        message: String,
    ) -> Result<teloxide::types::Message, teloxide::RequestError>
    where
        C: Into<teloxide::types::ChatId> + Send,
    {
        self.send_message(chat_id, message)
            .parse_mode(ParseMode::Html)
            .disable_notification(false)
            .send()
            .await
    }
}

#[async_trait]
impl NewsBot for teloxide::adaptors::AutoSend<teloxide::Bot> {
    async fn send_news<'a, C>(
        &self,
        chat_id: C,
        message: String,
    ) -> Result<teloxide::types::Message, teloxide::RequestError>
    where
        C: Into<teloxide::types::ChatId> + Send,
    {
        self.inner().send_news(chat_id, message).await
    }
}

pub struct NewsMessageBuilder<'a> {
    pub page: &'a NewsPage,
    pub news: &'a News,
    pub telegraph_page: &'a telegraph_rs::Page,
    pub tagger: &'a Tagger,
}

impl<'a> MessageBuilder for NewsMessageBuilder<'a> {
    fn build_message(&self) -> String {
        let (title, tags) = tags(self.page, self.tagger);

        let link = &self.telegraph_page.url;
        let id = self.news.id;
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

#[async_trait]
pub trait NewsClient: Sync {
    async fn news_get(&self, href: &str) -> Result<Response, Error>;

    fn news_list_href(&self, page: i32) -> String {
        format!("news?page={page}", page = page)
    }
    fn news_detail_href(&self, news_id: i32) -> String {
        format!("news/newsDetail/{news_id}", news_id = news_id)
    }

    async fn news_page(&self, news_id: i32) -> Result<NewsPage, Error> {
        let href = self.news_detail_href(news_id);
        let html = self.news_get(&href).await?.text().await?;

        NewsPage::from_html(html)
    }

    async fn news_page_from_href(&self, href: &str) -> Result<NewsPage, Error> {
        let html = self.news_get(&href).await?.text().await?;

        NewsPage::from_html(html)
    }

    async fn news_list_page(&self, page: i32) -> Result<NewsList, Error> {
        let href = self.news_list_href(page);
        let html = self.news_get(&href).await?.text().await?;

        NewsList::from_html(html)
    }

    async fn news_list(&self, page: i32) -> Result<Vec<News>, Error> {
        Ok(self.news_list_page(page).await?.news_list)
    }
}

#[async_trait::async_trait]
impl NewsClient for Client {
    async fn news_get(&self, href: &str) -> Result<Response, Error> {
        let url = self.news_server().join(href)?;
        let response = self.get(url).send().await?;
        Ok(response)
    }
}

pub fn tags<'a>(page: &'a NewsPage, tagger: &'a Tagger) -> (&'a str, Vec<String>) {
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
