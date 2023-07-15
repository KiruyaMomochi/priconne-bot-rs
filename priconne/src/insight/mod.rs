mod event;
pub mod tagging;

use std::fmt::Debug;

pub use event::{get_events, EventPeriod};

use chrono::{DateTime, FixedOffset, Utc};
use linked_hash_set::LinkedHashSet;

use mongodb::bson::{self, Bson};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;

use crate::resource::{
    announcement::{sources::AnnouncementSource, AnnouncementResponse},
    Announcement,
};

use self::tagging::RegexTagger;

/// Insight collected from an announcement.
#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AnnouncementInsight<E> {
    pub title: String,
    pub source: AnnouncementSource,
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
    /// Events in the announcement.
    /// The current design is save a different events vector for each announcement.
    /// The latest saved events will be used when building message.
    pub events: Vec<EventPeriod>,
    pub extra: E,
}

impl<E> AnnouncementInsight<E> {
    pub fn with_telegraph_url(mut self, url: String) -> Self {
        self.telegraph_url = Some(url);
        self
    }
}

impl<E> AnnouncementInsight<E>
where
    E: Serialize,
{
    pub fn into_bson(self) -> AnnouncementInsight<Bson> {
        AnnouncementInsight {
            title: self.title,
            source: self.source,
            id: self.id,
            url: self.url,
            tags: self.tags,
            create_time: self.create_time,
            update_time: self.update_time,
            telegraph_url: self.telegraph_url,
            events: self.events,
            extra: mongodb::bson::to_bson(&self.extra).unwrap(),
        }
    }
}

impl AnnouncementInsight<bson::Bson> {
    pub fn build_message(&self, post: &Announcement) -> String {
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

#[derive(Debug, Clone)]
pub struct Extractor {
    pub tagger: RegexTagger,
}

pub trait AnnouncementPage {
    type ExtraData: Serialize + DeserializeOwned + Debug + Send;

    fn title(&self) -> String;
    fn content(&self) -> kuchikiki::NodeRef;
    fn create_time(&self) -> Option<DateTime<FixedOffset>>;
    fn extra(&self) -> Self::ExtraData;
    fn tags(&self, tagger: &RegexTagger) -> LinkedHashSet<String> {
        tagger.tag_title(&self.title())
    }
    fn events(&self) -> Vec<EventPeriod> {
        get_events(&self.content().into_element_ref().unwrap())
    }
}

impl Extractor {
    /// Extract announcement insight and events from the response.
    pub fn extract_announcement<P: AnnouncementPage>(
        &self,
        response: &AnnouncementResponse<P>,
    ) -> AnnouncementInsight<P::ExtraData> {
        let page = &response.page;

        AnnouncementInsight::<P::ExtraData> {
            id: response.post_id,
            url: response.url.clone(),
            source: response.source.clone(),
            tags: page.tags(&self.tagger),
            title: page.title(),
            create_time: page.create_time().map(|t| t.with_timezone(&Utc)),
            update_time: page.create_time().map(|t| t.with_timezone(&Utc)),
            telegraph_url: None,
            events: page.events(),
            extra: page.extra(),
        }
    }
}

pub type Tags = LinkedHashSet<String>;
