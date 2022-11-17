use mongodb::{bson::{Document, doc}, options::FindOneOptions};
use priconne_core::map_titie;
use resource::{information::Announce, message::SentMessage, news::News};
use teloxide::types::Message;

use crate::{information::InformatonResult, news::NewsResult};

impl super::Db {
    pub async fn find_information_in_sent(
        &self,
        announce: &Announce,
    ) -> Result<Option<SentMessage>, mongodb::error::Error> {
        let collection = self.sent_messages();
        let filter = information_sent_filter(announce);

        collection
            .find_one(
                filter,
                FindOneOptions::builder()
                    .sort(doc! {"update_time": -1})
                    .build(),
            )
            .await
    }

    pub async fn update_information_in_sent(
        &self,
        sent: &SentMessage,
        announce: &Announce,
    ) -> Result<Option<SentMessage>, mongodb::error::Error> {
        let collection = self.sent_messages();
        
        collection
            .find_one_and_update(
                doc! {
                    "message_id": sent.message_id
                },
                doc! {
                    "$currentDate": {
                        "update_time": true,
                    },
                    "$set": {
                        "announce_id": announce.announce_id,
                    },
                },
                None,
            )
            .await
    }

    pub async fn upsert_information_in_sent(
        &self,
        sent_information: &InformatonResult,
        announce: &Announce,
        message: &teloxide::types::Message,
        telegraph_url: &str,
    ) -> Result<(), mongodb::error::Error> {
        self.upsert_announce(announce).await?;

        let collection = self.sent_messages();
        match sent_information {
            InformatonResult::None => {
                let sent_message = SentMessage {
                    announce_id: Some(announce.announce_id),
                    mapped_title: map_titie(&announce.title.title),
                    message_id: message.id,
                    news_id: None,
                    telegraph_url: telegraph_url.to_owned(),
                    update_time: chrono::Utc::now(),
                };
                collection.insert_one(sent_message, None).await?;
            }
            InformatonResult::SentNoAnnounce(found_sent) => {
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
                                "announce_id": announce.announce_id,
                                "message_id": message.id,
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
    
    pub async fn check_news_in_sent(&self, news: &News) -> Result<NewsResult, mongodb::error::Error> {
        if let Some(news) = self.check_news(news).await? {
            return Ok(NewsResult::News(news));
        }

        let found_sent = self.find_news_in_sent(news).await?;
        if let Some(found_sent) = found_sent {
            if found_sent.news_id.is_none() {
                self.upsert_news(news).await?;
                self.update_news_in_sent(&found_sent, news).await?;
                return Ok(NewsResult::SentNews(found_sent));
            }
            return Ok(NewsResult::SentNoNews(found_sent));
        }

        Ok(NewsResult::None)
    }
    
    pub async fn find_news_in_sent(
        &self,
        news: &News,
    ) -> Result<Option<SentMessage>, mongodb::error::Error> {
        let collection = self.sent_messages();
        let filter = news_sent_filter(news);

        collection
            .find_one(
                filter,
                FindOneOptions::builder()
                    .sort(doc! {"update_time": -1})
                    .build(),
            )
            .await
    }

    pub async fn update_news_in_sent(
        &self,
        sent: &SentMessage,
        news: &News,
    ) -> Result<Option<SentMessage>, mongodb::error::Error> {
        let collection = self.sent_messages();
        

        collection
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
            .await
    }

    pub async fn upsert_news_in_sent(
        &self,
        found_sent: &NewsResult,
        news: &News,
        message: &Message,
        telegraph_url: &str,
    ) -> Result<(), mongodb::error::Error> {
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

fn information_sent_filter(announce: &Announce) -> Document {
    let time = chrono::Utc::now() - chrono::Duration::hours(24);
    let mapped_title = &map_titie(&announce.title.title);
    doc! {
        "$or": [
            {
                "mapped_title": mapped_title,
                "announce_id": null,
                "update_time": {
                    "$gte": time
                }
            },
            {
                "mapped_title": mapped_title,
                "announce_id": announce.announce_id
            }
        ]
    }
}

fn news_sent_filter(news: &News) -> Document {
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
