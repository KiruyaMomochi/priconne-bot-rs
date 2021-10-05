use super::*;
use crate::message::{SentMessage, SentMessageDatabase};
use async_trait::async_trait;
use mongodb::{
    bson::{doc, Document},
    options::{FindOneAndReplaceOptions, FindOneOptions},
};
use priconne_core::map_titie;

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
pub(super) trait NewsDatabase: SentMessageDatabase {
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
                self.upsert_news(news).await?;
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
        let collection = self.sent_messages();
        let find_result = collection
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
        self.upsert_news(news).await?;

        let collection = self.sent_messages();
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
                collection.insert_one(sent_message, None).await?;
            }
            NewsResult::SentNoNews(found_sent) => {
                collection
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

impl NewsDatabase for mongodb::Database {
    fn news(&self) -> mongodb::Collection<News> {
        self.collection("news")
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
