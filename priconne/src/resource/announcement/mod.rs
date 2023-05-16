pub mod information;
pub mod news;

use crate::{
    insight::{AnnouncementPage, Tags, EventPeriod, AnnouncementInsight},
    service::resource::ResourceResponse, utils::map_title, message::Sendable,
};

use chrono::{DateTime, Utc};
use mongodb::bson;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_with::serde_as;

use super::{Resource, Region};

// This will finally replaces `SentMessage`.
#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Announcement {
    /// Post ID.
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
    /// [embedded]: https://www.mongodb.com/docs/manual/tutorial/model-embedded-one-to-many-relationships-between-documents/
    pub events: Vec<EventPeriod>,
    /// History post ID.
    pub history: Option<bson::oid::ObjectId>,
    /// Latest version
    pub latest_version: usize,
    /// Data in this announcement
    pub data: Vec<AnnouncementInsight<bson::Bson>>,
}

impl Announcement {
    pub fn new<E>(insight: AnnouncementInsight<E>, events: Vec<EventPeriod>) -> Self
    where
        E: Serialize + DeserializeOwned,
    {
        Self {
            id: bson::oid::ObjectId::new(),
            mapped_title: map_title(&insight.title),
            region: Region::TW,
            history: None,
            latest_version: 0,
            data: vec![insight.into_bson()],
            events,
        }
    }

    pub fn push<E>(&mut self, insight: AnnouncementInsight<E>, events: Vec<EventPeriod>)
    where
        E: Serialize + DeserializeOwned,
    {
        self.data.push(insight.into_bson());
        self.events = events;
    }
}

impl Sendable for Announcement {
    fn message(&self) -> crate::message::Message {
        let data = self.data.last().unwrap();
        let text = data.build_message(self);

        crate::message::Message {
            silent: false,
            text,
            image_src: None,
        }
    }
}

// pub trait AnnouncementResource = Announcement + Resource
// where
//     <Self as Resource>::Client:
//         AnnouncementClient<<Self as Resource>::Metadata, Page = <Self as Announcement>::Page>;

pub mod sources {
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
