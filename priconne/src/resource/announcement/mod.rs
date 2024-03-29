//! Announcements
//!
//! Client that can fetch announcements should implement [`AnnouncementClient`] trait.

pub mod event;
pub mod information;
pub mod news;
pub mod service;

use crate::{
    chat::Sendable,
    client::ResourceResponse,
    insight::{AnnouncementInsight, AnnouncementPage, EventInAnnouncement},
    utils::map_title,
};

use mongodb::bson;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;

use super::Region;

/// Announcement resource
#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Announcement {
    /// Announcement ID.
    /// Can generate by `bson::oid::ObjectId::new()`.
    #[serde(rename = "_id")]
    pub id: bson::oid::ObjectId,
    /// Mapped title for matching.
    pub mapped_title: String,
    /// Region of the post.
    pub region: Region,
    /// Events of post, will update when new data received.
    /// They are [embedded], sicne the number is small.
    ///
    /// Events are saved in both here and each [`AnnouncementInsight`],
    /// when [creating](Announcement::new) a new Announcement or a new
    /// insight is [pushed](Announcement::push), events in this struct
    /// will be updated (as for now, replaced).
    ///
    /// [embedded]: https://www.mongodb.com/docs/manual/tutorial/model-embedded-one-to-many-relationships-between-documents/
    pub events: Vec<EventInAnnouncement>,
    /// History post ID.
    pub history: Option<bson::oid::ObjectId>,
    /// Latest version
    pub latest_version: usize,
    /// Data in this announcement
    pub data: Vec<AnnouncementInsight<bson::Bson>>,
}

impl Announcement {
    pub fn new<E>(insight: AnnouncementInsight<E>, last: Option<Self>) -> Self
    where
        E: Serialize + DeserializeOwned,
    {
        match last {
            Some(mut last) => {
                last.push(insight);
                last
            }
            None => Self {
                id: bson::oid::ObjectId::new(),
                mapped_title: map_title(&insight.title),
                region: Region::TW,
                history: None,
                latest_version: 0,
                events: insight.events.clone(),
                data: vec![insight.into_bson()],
            },
        }
    }

    pub fn push<E>(&mut self, insight: AnnouncementInsight<E>)
    where
        E: Serialize + DeserializeOwned,
    {
        self.data.push(insight.into_bson());
    }
}

impl Sendable for Announcement {
    fn message(&self) -> crate::chat::Message {
        let data = self.data.last().unwrap();
        let text = data.build_message(self);

        crate::chat::Message {
            silent: false,
            text,
            image_src: None,
        }
    }
}

pub mod sources {
    use std::fmt::Display;

    use super::*;

    // When change serde representations,
    // also change convert::From impl
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
    #[serde(rename_all = "lowercase")]
    pub enum AnnouncementSource {
        Api(String),
        Website,
    }

    impl AnnouncementSource {
        pub fn name(&self) -> String {
            match self {
                AnnouncementSource::Api(_id) => "announce".to_string(),
                AnnouncementSource::Website => "news".to_string(),
            }
        }
    }

    impl std::convert::From<AnnouncementSource> for mongodb::bson::Bson {
        fn from(value: AnnouncementSource) -> Self {
            match value {
                AnnouncementSource::Api(id) => bson::bson!({{"announce"}: id}),
                AnnouncementSource::Website => bson::bson!("news"),
            }
        }
    }

    impl Display for AnnouncementSource {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                AnnouncementSource::Api(id) => write!(f, "{}: {id}", self.name()),
                _ => write!(f, "{}", self.name()),
            }
        }
    }
}

pub struct AnnouncementResponse<T: AnnouncementPage> {
    pub post_id: i32,
    pub source: sources::AnnouncementSource,
    pub url: url::Url,
    pub page: T,
}

impl<T: AnnouncementPage> ResourceResponse for AnnouncementResponse<T> {
    fn telegraph_content(&self, extra: Option<String>) -> Result<Option<String>, crate::Error> {
        let content_node = self.page.content();

        let attrs = content_node.as_element().unwrap().clone().attributes;
        tracing::trace!("optimizing {attrs:?}");
        let content_node = crate::utils::optimize_for_telegraph(content_node);

        let mut content = telegraph_rs::doms_to_nodes(content_node.children()).unwrap();
        if let Some(data_json) = extra {
            content.push(telegraph_rs::Node::NodeElement(telegraph_rs::NodeElement {
                tag: "br".to_string(),
                attrs: None,
                children: None,
            }));
            content.push(telegraph_rs::Node::NodeElement(telegraph_rs::NodeElement {
                tag: "br".to_string(),
                attrs: None,
                children: None,
            }));
            content.push(telegraph_rs::Node::NodeElement(telegraph_rs::NodeElement {
                tag: "code".to_string(),
                attrs: None,
                children: Some(vec![telegraph_rs::Node::Text(data_json)]),
            }));
        }

        let content = serde_json::to_string(&content)?;

        Ok(Some(content))
    }
}
