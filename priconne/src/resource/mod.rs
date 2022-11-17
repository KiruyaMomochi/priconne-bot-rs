pub mod article;
pub mod glossary;
pub mod same;
pub mod post;
pub mod cartoon;

pub use article::*;
use self::{information::Announce, news::News, cartoon::Thumbnail, post::PostSource};

pub trait Resource {
    type IdType;
    fn id(&self) -> Self::IdType;
    fn title(&self) -> &str;
    fn is_update(&self, other: &Self) -> bool;
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
}