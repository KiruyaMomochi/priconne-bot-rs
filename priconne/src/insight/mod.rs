mod event;
pub mod tagging;

pub use event::{get_events, EventPeriod};

use chrono::{DateTime, FixedOffset, Utc};
use linked_hash_set::LinkedHashSet;
use mongodb::bson;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_with::serde_as;

use crate::{
    database::Post,
    resource::post::{sources::Source, PostPageResponse},
};

use self::tagging::RegexTagger;

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PostInsight<E> {
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

impl<E> PostInsight<E>
where
    E: Serialize + DeserializeOwned,
{
    pub fn push_into(self, post: Option<Post>) -> Post {
        match post {
            Some(mut post) => {
                post.push(self);
                post
            }
            None => Post::new(self),
        }
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
    ) -> PostInsight<P::ExtraData> {
        let page = &response.page;
        PostInsight::<P::ExtraData> {
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

pub type Tags = LinkedHashSet<String>;
