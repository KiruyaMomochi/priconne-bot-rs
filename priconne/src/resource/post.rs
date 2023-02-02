use std::collections::HashMap;

use crate::{
    insight::{event::EventPeriod, PostData, PostPage},
    service::resource::{ResourceClient, ResourceService},
    Error, message::{PostMessage, Message},
};
use chrono::{DateTime, TimeZone};
use linked_hash_set::LinkedHashSet;
use mongodb::bson;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use teloxide::{
    payloads::SendMessageSetters,
    requests::{Request, Requester},
    types::{ParseMode, Recipient},
};

use self::sources::Source;

use super::{
    information::{Announce, InformationPage},
    same::map_titie,
    Resource,
};

pub mod sources {

    use super::*;

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
    #[serde(rename_all = "lowercase")]
    pub enum Source {
        Announce(String),
        News,
    }

    impl Source {
        pub fn name(&self) -> String {
            match self {
                Source::Announce(id) => format!("announce.{id}"),
                Source::News => "news".to_string(),
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Region {
    JP,
    EN,
    TW,
    CN,
    KR,
    TH,
}

pub struct PostPageResponse<T> {
    pub post_id: i32,
    pub source: Source,
    pub url: url::Url,
    pub page: T,
}

// This will finally replaces `SentMessage`.
#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Post {
    /// Post ID.
    /// Can generate by `bson::oid::ObjectId::new()`.
    #[serde(rename = "_id")]
    pub id: bson::oid::ObjectId,
    /// Mapped title for matching.
    pub mapped_title: String,
    /// Region of the post.
    pub region: Region,
    /// History post ID.
    pub history: Option<bson::oid::ObjectId>,
    /// Latest version
    pub latest_version: usize,
    /// Data in this post
    pub data: Vec<PostData<bson::Bson>>,
}

impl Post {
    pub fn new<E>(data: PostData<E>) -> Self
    where
        E: Serialize + for<'a> Deserialize<'a>,
    {
        let data = data.into_bson_extra();

        Self {
            id: bson::oid::ObjectId::new(),
            mapped_title: map_titie(&data.title),
            region: Region::TW,
            history: None,
            latest_version: 0,
            data: vec![data],
        }
    }

    pub fn push<E>(&mut self, data: PostData<E>)
    where
        E: Serialize + for<'a> Deserialize<'a>,
        {
            self.data.push(data.into_bson_extra())
        }
}

impl PostData<bson::Bson> {
    pub fn build_message(&self) -> String {
        // let (title, tags) = tags(&page, &self.tagger);
        let link = self.telegraph_url.as_ref().unwrap();
        let id = self.id;
        let create_time = self.create_time.map_or("".to_string(), |t| t.to_string());
        let events = &self.events;
        let tags = &self.tags;
        let title = if self.title.starts_with('【') {
            if let Some((_, title)) = self.title.split_once('】') {
                title
            } else {
                &self.title
            }
        } else {
            &self.title
        };

        let mut tag_str = String::new();

        for tag in tags {
            tag_str.push('#');
            tag_str.push_str(tag);
            tag_str.push(' ');
        }

        if !tag_str.is_empty() {
            tag_str.pop();
            tag_str.push('\n');
        }

        let mut event_str = String::new();

        for event in events {
            event_str.push_str("- ");
            event_str.push_str(&event.title);
            event_str.push_str(": \n   ");
            event_str.push_str(event.start.format("%m/%d %H:%M").to_string().as_str());
            event_str.push_str(" - ");
            event_str.push_str(event.end.format("%m/%d %H:%M").to_string().as_str());
            event_str.push('\n');
        }
        if !event_str.is_empty() {
            event_str.insert(0, '\n');
            event_str.push('\n');
        }

        let head = format!("{tag}<b>{title}</b>\n", tag = tag_str, title = title);

        let tail = format!(
            "{link}\n{time} <code>#{id}</code>",
            link = link,
            time = create_time,
            id = id
        );

        let message = format!("{}{}{}", head, event_str, tail);
        message
    }
}

impl PostMessage for Post {
    fn message(&self) -> Message {
        let data = self.data.last().unwrap();
        let text = data.build_message();

        Message {
            post_id: self.id,
            silent: false,
            text,
            results: vec![]
        }
    }
}