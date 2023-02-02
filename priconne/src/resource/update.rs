use std::fmt::Debug;

use tracing::debug;

use super::{
    post::{sources::Source, Post},
    Resource,
};

#[derive(Debug)]
pub struct ResourceFindResult<R: Resource> {
    /// this is new
    inner: R,
    /// this is a update to a existing item
    old: Option<R>,
    /// this is same as the one in database,
    is_same: bool,
}

impl<R: Resource> ResourceFindResult<R> {
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
pub struct ActionBuilder<'a, R: Resource> {
    pub source: &'a Source,
    /// Item in database before fetch
    pub resource: &'a ResourceFindResult<R>,
    /// Post item
    pub post: &'a Option<Post>,
}

/// Action to take
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

impl Action {
    pub fn need_full_article(&self) -> bool {
        match self {
            Action::None => false,
            Action::UpdateOnly => false,
            _ => true,
        }
    }

    pub fn is_none(&self) -> bool {
        if let Action::None = self {
            true
        } else {
            false
        }
    }

    pub fn is_update_only(&self) -> bool {
        if let Action::UpdateOnly = self {
            true
        } else {
            false
        }
    }
}

/// TODO: random write, may all wrong
impl<'a, R: Resource<IdType = i32> + Debug> ActionBuilder<'a, R> {
    pub fn new(
        source: &'a Source,
        resource: &'a ResourceFindResult<R>,
        post: &'a Option<Post>,
    ) -> Self {
        Self {
            source,
            resource,
            post,
        }
    }

    #[tracing::instrument]
    pub fn get_action(&'a self) -> Action {
        let source = self.source;
        let resource = &self.resource.inner;
        let post = self.post.as_ref();

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
            .rev().find(|p| &p.source == source && p.id == resource.id());
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
