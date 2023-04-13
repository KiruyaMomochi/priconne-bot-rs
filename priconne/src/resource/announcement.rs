use crate::{
    insight::{AnnouncementInsight, AnnouncementPage},
    message::{Message, Sendable},
    service::{resource::ResourceResponse, announcement::AnnouncementClient},
};

use mongodb::bson;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use self::sources::AnnouncementSource;

use super::Resource;

pub trait Announcement {
    type Page: AnnouncementPage;
    fn source(&self) -> AnnouncementSource;
}

pub trait AnnouncementResource = Announcement + Resource
where
    <Self as Resource>::Client:
        AnnouncementClient<<Self as Resource>::Metadata, Page = <Self as Announcement>::Page>;


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
                AnnouncementSource::Api(id) => "announce".to_string(),
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
    pub source: AnnouncementSource,
    pub url: url::Url,
    pub page: T,
}

impl<T> ResourceResponse for AnnouncementResponse<T>
where
    T: crate::insight::AnnouncementPage,
{
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
                children: Some(vec![telegraph_rs::Node::Text(data_json.to_string())]),
            }));
        }

        let content = serde_json::to_string(&content)?;

        Ok(Some(content))
    }
}
