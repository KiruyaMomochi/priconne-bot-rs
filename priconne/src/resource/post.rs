use crate::{
    insight::{PostInsight, PostPage},
    message::{Message, PostMessage},
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
