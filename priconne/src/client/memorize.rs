//! Helper types for [`MemorizedResourceClient`].
//!
//! [`FetchStrategy`] defines the strategy of fetching resources, which is used by
//! [`FetchState`] to determine whether a resource should be fetched.
//! [`MetadataFindResult`] is the resource metadata find result in database.

use futures::StreamExt;
use mongodb::bson::doc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::resource::ResourceMetadata;

/// Resource fetch strategy.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FetchStrategy {
    /// Stop fetch when continuous posted count is greater than this value.
    pub fuse_limit: Option<i32>,
    /// Minimum post id.
    pub ignore_id_lt: Option<i32>,
    /// Minimum update time,
    pub ignore_time_lt: Option<chrono::DateTime<chrono::Utc>>,
}

impl FetchStrategy {
    pub fn build(&self) -> FetchState<i32> {
        FetchState::new(self.clone())
    }
    pub fn override_by(self, rhs: &Self) -> Self {
        Self {
            fuse_limit: rhs.fuse_limit.or(self.fuse_limit),
            ignore_id_lt: rhs.ignore_id_lt.or(self.ignore_id_lt),
            ignore_time_lt: rhs.ignore_time_lt.or(self.ignore_time_lt),
        }
    }
}

impl FetchStrategy {
    pub const DEFAULT: Self = Self {
        fuse_limit: Some(1),
        ignore_id_lt: None,
        ignore_time_lt: None,
    };
}

impl Default for FetchStrategy {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// State of resource fetch.
#[derive(Debug, Clone)]
pub struct FetchState<I> {
    pub strategy: FetchStrategy,
    pub fuse_count: I,
}

impl FetchState<i32> {
    pub fn new(strategy: FetchStrategy) -> Self {
        Self {
            strategy,
            fuse_count: 0,
        }
    }

    pub fn keep_going<R: ResourceMetadata>(&mut self, resource: &R, is_update: bool) -> bool {
        let id = resource.id();
        let update_time = resource.update_time();

        let mut keep_going = true;
        if let Some(ignore_id_lt) = self.strategy.ignore_id_lt {
            if id < ignore_id_lt {
                keep_going = false;
            }
        }
        if let Some(ignore_time_lt) = self.strategy.ignore_time_lt {
            if update_time < ignore_time_lt {
                keep_going = false;
            }
        }
        if self.strategy.fuse_limit.is_none() {
            return keep_going;
        }

        if !is_update {
            self.fuse_count += 1;
        }
        if keep_going {
            self.fuse_count = 0;
        } else {
            self.fuse_count += 1;
        }

        let result = self.fuse_count < self.strategy.fuse_limit.unwrap_or(0);
        tracing::debug!(
            "id: {}/{:?}, fuse: {}/{:?}",
            id,
            self.strategy.ignore_id_lt,
            self.fuse_count,
            self.strategy.fuse_limit
        );
        result
    }

    pub fn should_fetch(&self) -> bool {
        match self.strategy.fuse_limit {
            Some(fuse_limit) => self.fuse_count < fuse_limit,
            None => true,
        }
    }
}

/// Find result of resource metadata.
/// Used by [`MemorizedResourceClient`] to determine whether to update the resource,
/// and return the old resource if it exists.
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
