use std::fmt::Debug;

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tracing::debug;

use crate::{
    database::{Announcement, AnnouncementCollection},
    insight::AnnouncementInsight,
    resource::{announcement::sources::AnnouncementSource, ResourceMetadata},
};

#[derive(Debug)]
pub struct MetadataFindResult<R: ResourceMetadata> {
    /// this is new
    inner: R,
    /// this is a update to a existing item
    old: Option<R>,
    /// this is same as the one in database,
    is_same: bool,
}

impl<R: ResourceMetadata> MetadataFindResult<R> {
    pub fn from_new(inner: R) -> Self {
        Self {
            inner,
            old: None,
            is_same: false,
        }
    }
    pub fn from_found(inner: R, old: R) -> Self {
        Self {
            is_same: !inner.is_update(&old),
            inner,
            old: Some(old),
        }
    }

    pub fn item(&self) -> &R {
        &self.inner
    }

    pub fn is_new(&self) -> bool {
        self.old.is_none()
    }

    pub fn is_update(&self) -> bool {
        !self.is_same && self.old.is_some()
    }

    pub fn is_same(&self) -> bool {
        self.is_same
    }

    pub fn is_not_same(&self) -> bool {
        !self.is_same
    }
}

/// Use information about a resource to find action to take
#[derive(Debug)]
pub struct AnnouncementDecision<R: ResourceMetadata> {
    /// Action to take
    pub action: Action,
    /// Source of item
    pub source: AnnouncementSource,
    /// Item in database before fetch
    pub resource: MetadataFindResult<R>,
    /// Post item
    pub announcement: Option<Announcement>,
}

/// Action to take
#[derive(Debug)]
pub enum Action {
    /// Do nothing, just return
    None,
    /// Update post, but do not send or edit message
    UpdateOnly,
    /// Update post and send message
    Send,
    /// Update post and edit message
    Edit,
}

impl<R: ResourceMetadata + Debug> AnnouncementDecision<R> {
    pub fn should_request(&self) -> Option<&R> {
        match self.action {
            Action::None => None,
            _ => Some(self.resource.item()),
        }
    }

    pub fn update_announcement<E>(&mut self, data: AnnouncementInsight<E>) -> Option<&Announcement>
    where
        E: Serialize + DeserializeOwned,
    {
        data.push_inplace(&mut self.announcement);
        self.announcement.as_ref()
    }

    pub fn send_post_and_continue(&self) -> Option<&Announcement> {
        match self.action {
            Action::Send => self.announcement.as_ref(),
            _ => None,
        }
    }
}

/// TODO: random write, may all wrong
impl<R: ResourceMetadata + Debug> AnnouncementDecision<R> {
    pub fn new(source: AnnouncementSource, find_result: MetadataFindResult<R>, announcement: Option<Announcement>) -> Self {
        Self {
            action: Self::get_action(&source, &find_result, &announcement),
            source,
            resource: find_result,
            announcement,
        }
    }

    fn get_action(
        source: &AnnouncementSource,
        resource: &MetadataFindResult<R>,
        post: &Option<Announcement>,
    ) -> Action {
        let resource = resource.item();
        if post.is_none() {
            debug!("old post is None. creating a new one");
            return Action::Send;
        }
        let post = post.as_ref().unwrap();

        // find resource with same source
        let same_source = post.data.iter().any(|p| &p.source == source);
        if !same_source {
            debug!("old post does not contain current source. update the post without posting");
            return Action::UpdateOnly;
        };

        // find resource with same id
        let found_resource = post
            .data
            .iter()
            .rev()
            .find(|p| &p.source == source && p.id == resource.id());
        if found_resource.is_none() {
            debug!("old post has a different source id. edit the old message");
            return Action::Edit;
        }
        let found_resource = found_resource.unwrap();

        let update_time = found_resource.update_time.or(found_resource.create_time);
        if update_time.is_some() && resource.update_time() > update_time.unwrap() {
            debug!("old post has same source, but current one is newer. edit the old message");
            return Action::Edit;
        }

        debug!("old post has same source, but current one is newer. edit the old message");
        Action::None
    }
}
