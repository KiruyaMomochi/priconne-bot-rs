pub mod announcement;
pub mod article;
pub mod cartoon;
pub mod glossary;

use crate::{
    insight::AnnouncementPage,
    service::{
        resource::{AnnouncementClient, ResourceClient, ResourceService},
        PriconneService,
    },
    utils::HOUR,
};
pub use article::*;
use chrono::{DateTime, FixedOffset, Utc};
use reqwest::{Client, Response};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use self::{
    announcement::{sources::AnnouncementSource, AnnouncementResponse},
    cartoon::Thumbnail,
    information::Announce,
    news::News,
};
use regex::Regex;

// pub enum Resource {
//     Announce,
//     News,
//     Cartoon,
// }

// impl Resource {
//     pub fn name(&self) -> &'static str {
//         match self {
//             Resource::Announce => "announce",
//             Resource::News => "news",
//             Resource::Cartoon => "cartoon",
//         }
//     }
// }

pub trait Resource {
    type Metadata: ResourceMetadata;
    type Client: ResourceClient<Self::Metadata>;

    fn name(&self) -> &'static str;
    fn build_service(
        &self,
        priconne: &PriconneService,
    ) -> ResourceService<Self::Metadata, Self::Client>;
    fn collection_name(&self) -> &'static str {
        self.name()
    }
}

pub trait Announcement {
    type Page: AnnouncementPage;
    fn source(&self) -> AnnouncementSource;
}

// pub trait BoundedAnnouncementResource: Announcement + Resource
// where
//     Self::Client:
//         ResourceClient<Self::Metadata, Response = AnnouncementResponse<Self::Page>>
//     // <Self as Resource>::Client:
//     //     AnnouncementClient<<Self as Resource>::Metadata, Page = <Self as Announcement>::Page>,
// {
// }

// impl<T> BoundedAnnouncementResource for T
// where
//     Self: Announcement + Resource<Metadata = <Self as Resource>::Metadata>,
//     <Self as Resource>::Client: AnnouncementClient<<Self as Resource>::Metadata>,
// {
// }

// pub trait AnnouncementResource: BoundedAnnouncementResource {}
// impl<T: BoundedAnnouncementResource> AnnouncementResource for T {}

pub trait AnnouncementResource = Announcement + Resource
where
    <Self as Resource>::Client:
        AnnouncementClient<<Self as Resource>::Metadata, Page = <Self as Announcement>::Page>;

/// Metadata for a resource
pub trait ResourceMetadata
where
    Self: std::fmt::Debug + Sync + Send + Unpin + Serialize + DeserializeOwned,
{
    fn id(&self) -> i32;
    fn title(&self) -> &str;
    fn update_time(&self) -> DateTime<Utc>;

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
mod tests {
    use super::*;
}
