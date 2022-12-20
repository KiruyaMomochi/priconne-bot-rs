use super::{
    information::InformationPage,
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
pub struct ActionBuilder<'a, R: Resource> {
    pub source: &'a Source,
    /// Item in database before fetch
    pub resource: &'a ResourceFindResult<R>,
    /// Post item
    pub post: &'a Option<Post>,
}

/// Action to take
pub enum Action {
    /// Do nothing
    None,
    /// Update database
    UpdateOnly,
    /// Edit an existing post
    Edit,
    /// Create and send a new post
    Create,
}

impl Action {
    pub fn need_full_article(&self) -> bool {
        match self {
            Action::Edit => true,
            Action::Create => true,
            _ => false,
        }
    }

    pub fn is_none(&self) -> bool {
        if let Action::None = self {
            true
        } else {
            false
        }
    }
}

/// TODO: random write, may all wrong
impl<'a, R: Resource<IdType = i32>> ActionBuilder<'a, R> {
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

    pub fn get_action(&'a self) -> Action {
        let source = self.source;
        let resource = self.resource.inner;
        let post = self.post.as_ref();

        if post.is_none() {
            return Action::Create;
        }
        let post = post.unwrap();

        let db_id = match post.sources.matches(source) {
            Some(db_id) => db_id,
            None => return Action::UpdateOnly,
        };

        if db_id != resource.id() {
            return Action::Edit;
        }

        if resource.update_time() > post.update_time {
            return Action::Edit;
        }

        Action::None
    }
}
