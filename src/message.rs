mod tagger;
pub use tagger::{map_titie, Tagger};

use async_trait::async_trait;
use chrono::DateTime;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

pub trait MessageBuilder {
    /// Builds a message
    fn build_message(&self) -> String;
}

#[async_trait]
pub(crate) trait SentMessageDatabase {
    fn sent_messages(&self) -> mongodb::Collection<SentMessage>;
}

impl SentMessageDatabase for mongodb::Database {
    fn sent_messages(&self) -> mongodb::Collection<SentMessage> {
        self.collection("sent_message")
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SentMessage {
    pub mapped_title: String,
    pub announce_id: Option<i32>,
    pub news_id: Option<i32>,
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub update_time: DateTime<chrono::Utc>,
    pub telegraph_url: String,
    pub message_id: i32,
}
