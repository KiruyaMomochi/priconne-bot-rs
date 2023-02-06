pub mod article;
pub mod cartoon;
pub mod glossary;
pub mod post;

use crate::utils::HOUR;
pub use article::*;
use chrono::{DateTime, FixedOffset, Utc};

use self::{cartoon::Thumbnail, information::Announce, news::News};
use regex::Regex;

pub trait Resource {
    type IdType;
    fn id(&self) -> Self::IdType;
    fn title(&self) -> &str;
    fn is_update(&self, other: &Self) -> bool;
    fn update_time(&self) -> DateTime<Utc>;
}

impl Resource for Announce {
    type IdType = i32;
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

impl Resource for News {
    type IdType = i32;
    fn is_update(&self, other: &Self) -> bool {
        self.id == other.id && (self.title != other.title || self.date > other.date)
    }
    fn id(&self) -> Self::IdType {
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

impl Resource for Thumbnail {
    type IdType = i32;
    fn is_update(&self, other: &Self) -> bool {
        self.id == other.id && (self.title != other.title || self.episode != other.episode)
    }
    fn id(&self) -> Self::IdType {
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

impl<T: Resource> Resource for &T {
    type IdType = T::IdType;
    fn is_update(&self, other: &Self) -> bool {
        T::is_update(self, other)
    }
    fn id(&self) -> Self::IdType {
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
