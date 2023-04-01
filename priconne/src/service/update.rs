use std::fmt::Debug;

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tracing::debug;

use crate::{
    database::{Post, PostCollection},
    insight::PostInsight,
    resource::{post::sources::Source, ResourceMetadata},
};

#[derive(Debug)]
pub struct ResourceFindResult<R: ResourceMetadata> {
    /// this is new
    inner: R,
    /// this is a update to a existing item
    old: Option<R>,
    /// this is same as the one in database,
    is_same: bool,
}

impl<R: ResourceMetadata> ResourceFindResult<R> {
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
pub struct Decision<R: ResourceMetadata> {
    /// Action to take
    pub action: Action,
    /// Source of item
    pub source: Source,
    /// Item in database before fetch
    pub resource: ResourceFindResult<R>,
    /// Post item
    pub post: Option<Post>,
}

/// Action to take
#[derive(Debug)]
enum Action {
    /// Do nothing, just return
    None,
    /// Update post, but do not send or edit message
    UpdateOnly,
    /// Update post and send message
    Send,
    /// Update post and edit message
    Edit,
}

impl<R: ResourceMetadata<IdType = i32> + Debug> Decision<R> {
    pub fn fetch_page_and_continue(&self) -> Option<&R> {
        match self.action {
            Action::None => None,
            _ => Some(self.resource.item()),
        }
    }

    pub fn update_post<E>(&mut self, data: PostInsight<E>) -> Option<&Post>
    where
        E: Serialize + DeserializeOwned,
    {
        self.post = Some(data.push_into(self.post));
        self.post.as_ref()
    }

    pub fn send_post_and_continue(&self) -> Option<&Post> {
        match self.action {
            Action::Send => self.post.as_ref(),
            _ => None,
        }
    }
}

/// TODO: random write, may all wrong
impl<R: ResourceMetadata<IdType = i32> + Debug> Decision<R> {
    pub fn new(source: Source, resource: ResourceFindResult<R>, post: Option<Post>) -> Self {
        Self {
            action: Self::get_action(&source, &resource, &post),
            source,
            resource,
            post,
        }
    }

    fn get_action(
        source: &Source,
        resource: &ResourceFindResult<R>,
        post: &Option<Post>,
    ) -> Action {
        let resource = resource.item();
        if post.is_none() {
            debug!("old post is None. creating a new one");
            return Action::Send;
        }
        let post = post.unwrap();

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
