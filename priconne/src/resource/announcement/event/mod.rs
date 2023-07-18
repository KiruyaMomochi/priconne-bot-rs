use chrono::{DateTime, Utc};
use mongodb::bson;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum EventKind {
    /// A campaign
    Campaign,
    /// A maintenance
    Maintenance,
    /// A new character
    Character,
    /// A new weapon
    Weapon,
    /// A new story
    Story,
    /// A new event
    Event,
    /// A new gacha
    Gacha,
    /// A new item
    Item,
    /// A new system
    System,
    /// A new feature
    Feature,
    /// A new other thing
    Other,
}

/// An event in announcement, or may also from other sources in the future?
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Event {
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub start: DateTime<Utc>,
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub end: DateTime<Utc>,
    pub title: String,
    pub announcement_title: String,
    pub announcement_id: bson::oid::ObjectId,
    pub kind: EventKind,
}
