use std::collections::HashMap;

use crate::{event::EventPeriod, service::resource::{ResourceClient, ResourceService}, Error};
use chrono::{DateTime, TimeZone};
use linked_hash_set::LinkedHashSet;
use mongodb::bson::{self, doc, oid};
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
// pub trait MessageBuilder {
//     /// Builds a message
//     fn build_message(&self) -> String;
// }

pub mod sources {

    use super::*;

    // #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
    // pub struct AnnounceSource {
    //     pub api: String,
    //     pub id: i32,
    // }
    // #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
    // pub struct NewsSource {
    //     pub id: i32,
    // }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
    #[serde(rename_all = "lowercase")]
    pub enum Source {
        Announce(String),
        News,
    }

    // impl From<AnnounceSource> for Source {
    //     fn from(announce: AnnounceSource) -> Self {
    //         Source::Announce(announce)
    //     }
    // }

    // impl From<NewsSource> for Source {
    //     fn from(news: NewsSource) -> Self {
    //         Source::News(news)
    //     }
    // }

    // impl From<&Announce> for Source {
    //     fn from(announce: &Announce) -> Self {
    //         Source::Announce(announce.clone())
    //     }
    // }

    // impl From<&News> for Source {
    //     fn from(news: &News) -> Self {
    //         Source::News(news.clone())
    //     }
    // }

    impl Source {
        pub fn name(&self) -> String {
            match self {
                Source::Announce(id) => format!("announce.{id}"),
                Source::News => "news".to_string(),
            }
        }

        pub fn to_sources(self, id: i32) -> PostSources {
            match self {
                Source::Announce(api_id) => PostSources::new_announce(api_id, id),
                Source::News => PostSources::new_news(id),
            }
        }

        // pub fn bson(&self) -> bson::Bson {
        //     match self {
        //         Source::Announce(announce) => bson::to_bson(announce).unwrap(),
        //         Source::News(news) => bson::to_bson(news).unwrap(),
        //     }
        // }
    }
}

// #[derive(Debug)]
// pub enum PostKind {
//     Announce(Announce, String),
//     News(News),
// }

// impl PostKind {
//     pub fn into_source(&self) -> sources::Source {
//         match self {
//             PostKind::Announce(announce, api) => sources::AnnounceSource{api: api.to_string(), id: announce.announce_id}.into(),
//             PostKind::News(news) => sources::NewsSource{id: news.id}.into(),
//         }
//     }

//     pub fn title(&self) -> String {
//         match self {
//             PostKind::Announce(announce, _) => announce.title.title.to_string(),
//             PostKind::News(news) => news.title.to_string(),
//         }
//     }
// }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PostSources {
    pub announce: HashMap<String, i32>,
    pub news: Option<i32>,
}

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
// pub struct PostSources(pub Vec<sources::Source>);
// pub fn has_announce(&self) -> bool {
//     self.0.iter().any(|s| {
//         if let sources::Source::Announce(_) = s {
//             true
//         } else {
//             false
//         }
//     })
// }

// pub fn has_news(&self) -> bool {
//     self.0.iter().any(|s| {
//         if let sources::Source::News(_) = s {
//             true
//         } else {
//             false
//         }
//     })
// }

// pub fn announce(&mut self) -> Option<&mut sources::Announce> {
//     self.0.iter_mut().find_map(|s| {
//         if let sources::Source::Announce(announce) = s {
//             Some(announce)
//         } else {
//             None
//         }
//     })
// }

// pub fn upsert_announce(&mut self, announce: sources::Announce) {
//     if let Some(announce_index) = self.0.iter().position(|s| {
//         if let sources::Source::Announce(_) = s {
//             true
//         } else {
//             false
//         }
//     }) {
//         self.0[announce_index] = announce.into();
//     } else {
//         self.0.push(announce.into());
//     }
// }

impl PostSources {
    // pub fn new(sources: Vec<sources::Source>) -> Self {
    //     let mut announce = HashMap::new();
    //     let mut news = Vec::new();
    //     for source in sources {
    //         match source {
    //             sources::Source::Announce(announcement) => {
    //                 announce.push(announcement)
    //             }
    //             sources::Source::News(news_) => {
    //                 news.push(news_)
    //             }
    //         }
    //     }

    //     Self {
    //         announce,
    //         news,
    //     }
    // }

    pub fn has_announce(&self) -> bool {
        !self.announce.is_empty()
    }

    pub fn has_news(&self) -> bool {
        self.news.is_some()
    }

    // pub fn announce(&mut self) -> Option<(&String, &mut i32)> {
    //     self.announce.iter_mut().next()
    // }

    pub fn new_news(id: i32) -> Self {
        Self {
            announce: HashMap::new(),
            news: Some(id),
        }
    }

    pub fn new_announce(api_id: String, id: i32) -> Self {
        Self {
            announce: HashMap::from([(api_id, id)]),
            news: None,
        }
    }

    // pub fn replace_announce(&mut self, announce: sources::AnnounceSource) {
    //     self.announce = vec![announce];
    // }
}

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
// #[serde(tag = "type")]
// pub enum PostSource {
//     Announce { api: String, id: i32 },
//     News { id: i32 },
// }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Region {
    JP,
    EN,
    TW,
    CN,
    KR,
    TH,
}

// This will finally replaces `SentMessage`.
#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Post {
    /// Post ID.
    /// Can generate by `bson::oid::ObjectId::new()`.
    #[serde(rename = "_id")]
    pub id: oid::ObjectId,
    /// The title of the post.
    pub title: String,
    /// Mapped title for matching.
    pub mapped_title: String,
    /// Region of the post.
    pub region: Region,
    /// Source of the post.
    pub source: PostSources,
    /// The time when the post was created.
    #[serde_as(as = "mongodb::bson::DateTime")]
    pub create_time: DateTime<chrono::Utc>,
    /// The time when the post was updated.
    #[serde_as(as = "mongodb::bson::DateTime")]
    pub update_time: DateTime<chrono::Utc>,
    /// History post ID.
    pub history: Option<oid::ObjectId>,
    /// Tags of the post.
    pub tags: LinkedHashSet<String>,
    /// Events contained in the post.
    pub events: Vec<EventPeriod>,
    /// Telegraph page URL.
    pub telegraph: Option<String>,
    /// Message ID in chat.
    pub message_id: Option<i32>,
}

// #[serde_as]
#[derive(Clone, Debug)]
pub struct NewMessage {
    /// The title of the post.
    pub title: String,
    /// Display title
    pub display_title: String,
    /// Source of the post.
    pub source: Source,
    /// Id of the post.
    pub id: i32,
    /// The time when the post was created.
    pub create_time: DateTime<chrono::Utc>,
    /// History post ID.
    pub history: Option<oid::ObjectId>,
    /// Tags of the post.
    pub tags: LinkedHashSet<String>,
    /// Events contained in the post.
    pub events: Vec<EventPeriod>,
    /// Telegraph page URL.
    pub telegraph: Option<String>,
    /// Silent?
    pub silent: bool,
}

impl NewMessage {
    pub fn build_message(&self) -> String {
        // let (title, tags) = tags(&page, &self.tagger);
        let link = self.telegraph.as_ref().unwrap();
        let id = self.id;
        let time = self.create_time;
        let events = &self.events;
        let tags = &self.tags;
        let title = &self.display_title;

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
            time = time,
            id = id
        );

        let message = format!("{}{}{}", head, event_str, tail);
        message
    }

    pub async fn send(self, bot: teloxide::Bot, chat_id: Recipient) -> Result<Post, Error> {
        let text = self.build_message();
        let send_result = bot
            .send_message(chat_id, text)
            .parse_mode(ParseMode::Html)
            .disable_notification(self.silent)
            .send()
            .await?;

        Ok(Post {
            id: bson::oid::ObjectId::new(),
            mapped_title: map_titie(&self.title),
            title: self.title,
            region: Region::TW,
            source: self.source.to_sources(self.id),
            create_time: self.create_time,
            update_time: self.create_time,
            history: self.history,
            tags: self.tags,
            events: self.events,
            telegraph: self.telegraph,
            message_id: Some(send_result.id),
        })
    }
}

impl Post {
    pub fn dummy() -> Self {
        let time = chrono::Utc.timestamp(0, 0);
        Self {
            id: oid::ObjectId::new(),
            mapped_title: "這不可能出現在頻道中".to_string(),
            title: "這不可能出現在頻道中".to_string(),
            region: Region::TW,
            source: PostSources {
                announce: HashMap::new(),
                news: None,
            },
            create_time: time,
            update_time: time,
            history: None,
            tags: LinkedHashSet::new(),
            events: vec![],
            telegraph: None,
            message_id: None,
        }
    }

    pub fn new(
        title: String,
        tags: LinkedHashSet<String>,
        time: DateTime<chrono::Utc>,
        sources: PostSources,
        events: Vec<EventPeriod>,
        telegraph: String,
        // message_id: i32,
    ) -> Self {
        // let time = create_time.map_or(chrono::Utc.timestamp(0, 0), |t| {
        //     t.with_timezone(&chrono::Utc)
        // });
        Self {
            id: oid::ObjectId::new(),
            mapped_title: map_titie(&title),
            title,
            region: Region::TW,
            source: sources,
            create_time: time,
            update_time: time,
            history: None,
            tags,
            events,
            telegraph: Some(telegraph),
            message_id: None,
        }
    }

    // pub fn from_information(
    //     information: &super::article::information::InformationPage,
    //     telegraph_url: &str,
    //     api_name: &str,
    //     api_id: i32,
    // ) -> Self {
    //     let mut tags = LinkedHashSet::new();
    //     if let Some(icon) = information.icon {
    //         tags.insert(icon.to_tag().to_string());
    //     }
    //     for tag in crate::extract_tag(&information.title) {
    //         tags.insert_if_absent(tag);
    //     }

    //     Self::new(
    //         information.title.clone(),
    //         tags,
    //         information.date.map(|d| d.with_timezone(&chrono::Utc)),
    //         vec![sources::AnnounceSource {
    //             api: api_name.to_string(),
    //             id: api_id,
    //         }
    //         .into()],
    //         information.events.clone(),
    //         telegraph_url.to_string(),
    //     )
    // }

    // pub fn from_announce(
    //     announce: &Announce,
    //     information: &InformationPage,
    //     api_id: String,
    //     telegraph: &telegraph_rs::Page,
    // ) -> NewMessage {
    //     let mut tags = LinkedHashSet::new();
    //     Self::insert_tags_announce(&mut tags, information);

    //     let time = information
    //         .date
    //         .map_or(announce.replace_time, |d| d.with_timezone(&chrono::Utc));

    //     NewMessage {
    //         title: information.title.clone(),
    //         source: Source::Announce(api_id),
    //         id: announce.announce_id,
    //         create_time: time,
    //         history: None,
    //         tags,
    //         events: information.events,
    //         telegraph: Some(telegraph.url.to_string()),
    //         silent: false,
    //     }

    //     // Self::new(
    //     //     information.title.clone(),
    //     //     tags,
    //     //     time,
    //     //     PostSources {
    //     //         announce: HashMap::from([(api_id, announce.announce_id)]),
    //     //         news: None,
    //     //     },
    //     //     information.events.clone(),
    //     //     telegraph.to_string(),
    //     // )
    // }

    fn insert_tags_announce(tags: &mut LinkedHashSet<String>, information: &InformationPage) {
        if let Some(icon) = information.icon {
            tags.insert_if_absent(icon.to_tag().to_string());
        }
        for tag in crate::extract_tag(&information.title) {
            tags.insert_if_absent(tag);
        }
    }

    pub fn update_announce(
        &mut self,
        announce: Announce,
        information: &InformationPage,
        api_id: String,
        telegraph_url: &str,
    ) {
        self.source.announce.insert(api_id, announce.announce_id);
        Self::insert_tags_announce(&mut self.tags, information);

        self.update_time = information
            .date
            .map_or(announce.replace_time, |d| d.with_timezone(&chrono::Utc));

        self.telegraph = Some(telegraph_url.to_string());
        self.title = information.title.clone();
        self.mapped_title = map_titie(&self.title);
        self.events = information.events.clone();
    }
}

pub trait PostSource<R>
where
    R: Resource<IdType = i32>,
{
    fn post_source(&self) -> Source;
}

impl<T: PostSource<R>, R> PostSource<R> for &T
where
    R: Resource<IdType = i32>,
{
    fn post_source(&self) -> Source {
        T::post_source(self)
    }
}

// If it 

// pub trait PostService<R, C>
// where
//     Self: ResourceService<R, C>,
//     C: ResourceClient<R>,
//     R: Resource<IdType = i32>,
//     {

//     }