use crate::{
    insight::{PostInsight, PostPage},
    message::{Message, PostMessage},
    service::resource::ResourceResponse,
};

use mongodb::bson;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use self::sources::Source;

pub mod sources {
    use super::*;

    // When change serde representations,
    // also change convert::From impl
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
    #[serde(rename_all = "lowercase")]
    pub enum Source {
        Announce(String),
        News,
    }

    impl Source {
        pub fn name(&self) -> String {
            match self {
                Source::Announce(id) => "announce".to_string(),
                Source::News => "news".to_string(),
            }
        }
    }

    impl std::convert::From<Source> for mongodb::bson::Bson {
        fn from(value: Source) -> Self {
            match value {
                Source::Announce(id) => bson::bson!({{"announce"}: id}),
                Source::News => bson::bson!("news"),
            }
        }
    }
}

pub struct PostPageResponse<T> {
    pub post_id: i32,
    pub source: Source,
    pub url: url::Url,
    pub page: T,
}

impl<T> ResourceResponse for PostPageResponse<T>
where
    T: crate::insight::PostPage,
{
    fn title(&self) -> String {
        self.page.title()
    }

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
