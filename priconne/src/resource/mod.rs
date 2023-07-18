//! Resource
//!
//! This module defines various resources in Priconne world, such as
//! announcements, cartoons, etc.
//!
//! XXX: For [`ResourceId`] Here is a not-so-good design:
//! > What does our "Resource" mean?
//!
//! As [`ResourceId`] is mainly for cross-referencing in [`SendResult`](crate::message::SendResult),
//! we use [`announcement::Announcement`] instead of detailed resource there because
//! each message represents a full announcement.
//!
//! However, currently [`News`] and [`Announce`] are definitely a resource too, as
//! they are fetched from remote and stored in database. We need to find a way to
//! correctly define what is a resource and what is not.

pub mod announcement;
pub mod api;
pub mod cartoon;
pub mod glossary;
use std::fmt::Display;

pub use announcement::*;
use mongodb::bson;

use crate::{
    client::{MemorizedResourceClient, ResourceClient},
    service::PriconneService,
    utils::HOUR,
};
use chrono::{DateTime, FixedOffset, Utc};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use cartoon::Thumbnail;
use information::Announce;
use news::News;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Region {
    JP,
    /// No more
    EN,
    TW,
    CN,
    KR,
    TH,
}

/// Identifiers for resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceId {
    Announcement(bson::oid::ObjectId),
    Cartoon(i32),
}

/// Kind of a resource, the difference from [`ResourceId`] is that
/// this type does not have any fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceKind {
    Announce,
    News,
    Cartoon,
    Unknown,
}

impl Display for ResourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ResourceKind::Announce => "announce",
            ResourceKind::News => "news",
            ResourceKind::Cartoon => "cartoon",
            ResourceKind::Unknown => return Err(std::fmt::Error),
        };
        write!(f, "{}", s)
    }
}

/// Metadata for a resource
pub trait ResourceMetadata
where
    Self: std::fmt::Debug + Sync + Send + Unpin + Serialize + DeserializeOwned,
{
    fn id(&self) -> i32;
    fn title(&self) -> &str;
    fn update_time(&self) -> DateTime<Utc>;

    /// The [`ResourceKind`] for this client
    /// This is required to not use a plain literial string
    /// when creating connection to memorize database.
    /// May have better implementation.
    fn kind(&self) -> ResourceKind {
        ResourceKind::Unknown
    }

    // TODO: change to `compare` and return `Ordering` instead
    fn is_update(&self, other: &Self) -> bool;
}

impl ResourceMetadata for Announce {
    fn is_update(&self, other: &Announce) -> bool {
        self.announce_id == other.announce_id
            && (self.title != other.title || self.replace_time > other.replace_time)
    }
    fn id(&self) -> i32 {
        self.announce_id
    }
    fn title(&self) -> &str {
        &self.title.title
    }
    fn update_time(&self) -> DateTime<Utc> {
        self.replace_time
    }
}

impl ResourceMetadata for News {
    fn is_update(&self, other: &Self) -> bool {
        self.id == other.id && (self.title != other.title || self.date > other.date)
    }
    fn id(&self) -> i32 {
        self.id
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn update_time(&self) -> DateTime<Utc> {
        self.date
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(FixedOffset::east_opt(8 * HOUR).unwrap())
            .unwrap()
            .with_timezone(&Utc)
    }
}

impl ResourceMetadata for Thumbnail {
    fn is_update(&self, other: &Self) -> bool {
        self.id == other.id && (self.title != other.title || self.episode != other.episode)
    }
    fn id(&self) -> i32 {
        self.id
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn update_time(&self) -> DateTime<Utc> {
        // TODO
        Utc::now()
    }
}

impl<'a, T: ResourceMetadata> ResourceMetadata for &'a T
where
    &'a T: for<'de> Deserialize<'de>,
{
    fn is_update(&self, other: &Self) -> bool {
        T::is_update(self, other)
    }
    fn id(&self) -> i32 {
        T::id(self)
    }
    fn title(&self) -> &str {
        T::title(self)
    }

    fn update_time(&self) -> DateTime<Utc> {
        T::update_time(self)
    }
}

#[cfg(test)]
mod tests {}
