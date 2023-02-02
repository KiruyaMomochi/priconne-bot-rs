pub mod event;
pub mod tagging;

use chrono::{DateTime, FixedOffset, Utc};
use linked_hash_set::LinkedHashSet;
use mongodb::bson;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::resource::post::{sources::Source, PostPageResponse};

use self::{
    event::{get_events, EventPeriod},
    tagging::RegexTagger,
};

type Tags = LinkedHashSet<String>;

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PostData<E>
{
    pub title: String,
    pub source: Source,
    pub id: i32,
    pub url: url::Url,
    pub tags: Tags,
    pub events: Vec<EventPeriod>,
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

impl<E: Serialize> PostData<E>
{
    pub fn into_bson_extra(self) -> PostData<bson::Bson> {
        PostData {
            source: self.source,
            id: self.id,
            url: self.url,
            tags: self.tags,
            events: self.events,
            title: self.title,
            create_time: self.create_time,
            update_time: self.update_time,
            telegraph_url: None,
            extra: bson::to_bson(&self.extra).unwrap(),
        }
    }
}

impl<E> PostData<E>
{
    pub fn with_telegraph_url(mut self, url: String) -> Self {
        self.telegraph_url = Some(url);
        self
        
    }
}

pub struct Extractor {
    pub tagger: RegexTagger,
}

pub trait PostPage {
    type ExtraData;

    fn title(&self) -> String;
    fn content(&self) -> kuchiki::NodeRef;
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
    pub fn extract_post<P: PostPage>(
        &self,
        response: &PostPageResponse<P>,
    ) -> PostData<P::ExtraData> {
        let page = &response.page;
        PostData::<P::ExtraData> {
            id: response.post_id,
            url: response.url.clone(),
            source: response.source.clone(),
            tags: page.tags(&self.tagger),
            events: page.events(),
            title: page.title(),
            create_time: page.create_time().map(|t| t.with_timezone(&Utc)),
            update_time: page.create_time().map(|t| t.with_timezone(&Utc)),
            telegraph_url: None,
            extra: page.extra(),
        }
    }
}
