use chrono::{DateTime, FixedOffset, Utc};
use mongodb::bson;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_with::serde_as;

use crate::{
    insight::{EventPeriod, PostInsight, Tags},
    message::PostMessage,
    resource::post::{sources::Source, PostPageResponse, self},
    service::Region,
};

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PostData<E> {
    pub title: String,
    pub source: Source,
    pub id: i32,
    pub url: url::Url,
    pub tags: Tags,
    // Mongo can only save UTC time, it's our own's responsible to convert it to UTC+8
    // https://www.mongodb.com/docs/v4.4/tutorial/model-time-data/
    /// The time when the post was created.
    #[serde_as(as = "Option<mongodb::bson::DateTime>")]
    pub create_time: Option<DateTime<Utc>>,
    /// The time when the post was updated.
    #[serde_as(as = "Option<mongodb::bson::DateTime>")]
    pub update_time: Option<DateTime<Utc>>,
    pub telegraph_url: Option<String>,
    pub extra: E,
}

impl<E> PostData<E> {
    pub fn with_telegraph_url(mut self, url: String) -> Self {
        self.telegraph_url = Some(url);
        self
    }
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
    /// Events of post, will update when new data received.
    /// They are [embedded], sicne the number is small.
    ///
    /// [embedded]: https://www.mongodb.com/docs/manual/tutorial/model-embedded-one-to-many-relationships-between-documents/
    pub events: Vec<EventPeriod>,
    /// History post ID.
    pub history: Option<bson::oid::ObjectId>,
    /// Latest version
    pub latest_version: usize,
    /// Data in this post
    pub data: Vec<PostData<bson::Bson>>,
}

fn post_insight_to_data<E>(insight: PostInsight<E>) -> (PostData<bson::Bson>, Vec<EventPeriod>)
where
    E: Serialize + DeserializeOwned,
{
    (
        PostData {
            create_time: insight.create_time,
            id: insight.id,
            source: insight.source,
            tags: insight.tags,
            telegraph_url: insight.telegraph_url,
            title: insight.title,
            update_time: insight.update_time,
            url: insight.url,
            extra: bson::to_bson(&insight.extra).unwrap(),
        },
        insight.events,
    )
}

impl Post {
    pub fn new<E>(data: PostInsight<E>) -> Self
    where
        E: Serialize + DeserializeOwned,
    {
        let (data, events) = post_insight_to_data(data);
        Self {
            id: bson::oid::ObjectId::new(),
            mapped_title: super::map_title(&data.title),
            region: Region::TW,
            history: None,
            latest_version: 0,
            data: vec![data],
            events,
        }
    }

    pub fn push<E>(&mut self, data: PostInsight<E>)
    where
        E: Serialize + DeserializeOwned,
    {
        let (data, events) = post_insight_to_data(data);

        self.data.push(data);
        self.events = events;
    }
}

impl PostData<bson::Bson> {
    pub fn build_message(&self, post: &Post) -> String {
        // let (title, tags) = tags(&page, &self.tagger);
        let link = self.telegraph_url.as_ref().map_or("#NOURL", |s| s.as_str());
        let id = self.id;
        let create_time = self.create_time.map_or("".to_string(), |t| t.to_string());
        let events = &post.events;
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

        let head = format!("{tag_str}<b>{title}</b>\n");

        let tail = format!("{link}\n{create_time} <code>#{id}</code>");

        let message = format!("{head}{event_str}{tail}");
        message
    }
}

impl PostMessage for Post {
    fn message(&self) -> crate::message::Message {
        let data = self.data.last().unwrap();
        let text = data.build_message(self);

        crate::message::Message {
            post_id: self.id,
            silent: false,
            text,
            results: vec![],
        }
    }
}
