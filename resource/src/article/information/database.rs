use super::*;
use crate::message::{SentMessage, SentMessageDatabase};
use async_trait::async_trait;
use mongodb::{
    bson::{doc, Document},
    options::{FindOneAndReplaceOptions, FindOneOptions},
};
use priconne_core::map_titie;

pub enum InformatonResult {
    Announce(Announce),
    SentAnnounce(SentMessage),
    SentNoAnnounce(SentMessage),
    None,
}

impl InformatonResult {
    pub fn is_found(&self) -> bool {
        match self {
            InformatonResult::Announce(_) => true,
            InformatonResult::SentAnnounce(_) => true,
            InformatonResult::SentNoAnnounce(_) => false,
            InformatonResult::None => false,
        }
    }

    pub fn is_not_found(&self) -> bool {
        return !self.is_found();
    }
}

#[async_trait]
pub(super) trait InformationDatabase: SentMessageDatabase {
    fn announces(&self) -> mongodb::Collection<Announce>;

    async fn check_announce(
        &self,
        announce: &Announce,
    ) -> Result<Option<Announce>, mongodb::error::Error> {
        let found_announce = self.find_announce(announce).await?;
        if let Some(found_announce) = found_announce {
            if found_announce.replace_time == announce.replace_time {
                return Ok(Some(found_announce));
            }
        }
        return Ok(None);
    }

    async fn check_sent_announce(
        &self,
        announce: &Announce,
    ) -> Result<InformatonResult, mongodb::error::Error> {
        if let Some(announce) = self.check_announce(announce).await? {
            return Ok(InformatonResult::Announce(announce));
        }

        let found_sent = self.find_sent_information(announce).await?;
        if let Some(found_sent) = found_sent {
            if found_sent.update_time > announce.replace_time {
                self.upsert_announce(announce).await?;
                self.update_sent_information(&found_sent, announce).await?;
                return Ok(InformatonResult::SentAnnounce(found_sent));
            }
            return Ok(InformatonResult::SentNoAnnounce(found_sent));
        }

        Ok(InformatonResult::None)
    }

    async fn find_announce(
        &self,
        announce: &Announce,
    ) -> Result<Option<Announce>, mongodb::error::Error> {
        let collection = self.announces();
        let filter = announce_filter(announce);
        let find_result = collection.find_one(filter.clone(), None).await?;
        Ok(find_result)
    }

    async fn upsert_announce(
        &self,
        announce: &Announce,
    ) -> Result<Option<Announce>, mongodb::error::Error> {
        let collection = self.announces();
        let filter = announce_filter(announce);
        let options = Some(FindOneAndReplaceOptions::builder().upsert(true).build());
        let replace_result = collection
            .find_one_and_replace(filter, announce, options)
            .await?;
        Ok(replace_result)
    }

    async fn find_sent_information(
        &self,
        announce: &Announce,
    ) -> Result<Option<SentMessage>, mongodb::error::Error> {
        let collection = self.sent_messages();
        let filter = sent_filter(announce);
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

    async fn update_sent_information(
        &self,
        sent: &SentMessage,
        announce: &Announce,
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
                        "announce_id": announce.announce_id,
                    },
                },
                None,
            )
            .await;

        find_result
    }

    async fn upsert_sent_information(
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
}

fn announce_filter(announce: &Announce) -> Document {
    doc! {
        "announce_id": announce.announce_id,
    }
}

fn sent_filter(announce: &Announce) -> Document {
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

impl InformationDatabase for mongodb::Database {
    fn announces(&self) -> mongodb::Collection<Announce> {
        self.collection("announce")
    }
}
