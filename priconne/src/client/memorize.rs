//! [`MemorizedResourceClient`] and helper types.
//!
//! [`FetchStrategy`] defines the strategy of fetching resources, which is used by
//! [`FetchState`] to determine whether a resource should be fetched.
//! [`MetadataFindResult`] is the resource metadata find result in database.

use super::{ResourceClient, ResourceMetadataCollection};
use crate::{resource::ResourceMetadata, Error};
use async_trait::async_trait;
use futures::{future, stream::BoxStream, TryStreamExt};
use mongodb::{bson::doc, Collection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Wrapped `ResourceClient` that memorizes the fetched resource metadata.
pub struct MemorizedResourceClient<M, Client>
where
    Client: ResourceClient<M>,
    M: ResourceMetadata,
{
    pub client: Client,
    pub strategy: FetchStrategy,
    pub collection: ResourceMetadataCollection<M>,
}

impl<M, Client> MemorizedResourceClient<M, Client>
where
    Client: ResourceClient<M>,
    M: ResourceMetadata,
{
    pub(super) fn new(client: Client, strategy: FetchStrategy, collection: Collection<M>) -> Self {
        Self {
            client,
            strategy,
            collection: ResourceMetadataCollection::new(collection),
        }
    }

    async fn updated(&self, item: M) -> Result<MetadataFindResult<M>, Error> {
        let in_db = self.collection.find(&item).await?;

        let update = match in_db {
            Some(in_db) => MetadataFindResult::from_found(item, in_db),
            None => MetadataFindResult::from_new(item),
        };

        Ok(update)
    }

    pub(crate) async fn upsert(&self, item: &M) -> Result<Option<M>, Error> {
        self.collection.upsert(item).await.map_err(Error::from)
    }

    /// Fetches all resources and their state in the database, as a stream.
    fn compared_stream<'stream>(&'stream self) -> BoxStream<Result<MetadataFindResult<M>, Error>>
    where
        Self: Sync,
        M: 'stream,
    {
        let result = self
            .try_stream()
            .and_then(|item| self.updated(item))
            .into_stream();

        Box::pin(result)
    }

    /// Fetches resources that are new or updated in the database, as a stream.
    pub fn fused_stream<'stream>(&'stream self) -> BoxStream<Result<MetadataFindResult<M>, Error>>
    where
        Self: Sync,
        M: Send + 'stream,
    {
        let mut fetch_state = self.strategy.build();
        let result = self
            .compared_stream()
            .try_take_while(move |update| {
                let keep_going = fetch_state.keep_going(update);
                tracing::trace!(
                    "id = {}, new: {}, update: {}. keep_going: {keep_going}",
                    update.item().id(),
                    update.is_new(),
                    update.is_update()
                );
                future::ok(keep_going)
            })
            .try_filter(|update| future::ready(update.is_not_same()));

        Box::pin(result)
    }

    /// Fetches resources that are new or updated in the database, as a vector.
    pub async fn latests(&self) -> Result<Vec<MetadataFindResult<M>>, Error>
    where
        M: Send,
    {
        let stream = self.fused_stream();
        let result: Vec<_> = stream.try_collect().await?;

        let result = result.into_iter().rev().collect();

        Ok(result)
    }
}

#[async_trait]
impl<M, Client> ResourceClient<M> for MemorizedResourceClient<M, Client>
where
    Client: ResourceClient<M>,
    M: ResourceMetadata,
{
    type Response = Client::Response;
    fn try_stream(&self) -> BoxStream<Result<M, Error>> {
        self.client.try_stream()
    }
    async fn get_by_id(&self, id: i32) -> Result<Self::Response, Error> {
        self.client.get_by_id(id).await
    }
}

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

    pub fn keep_going<M: ResourceMetadata>(&mut self, found: &MetadataFindResult<M>) -> bool {
        let metadata = found.item();
        let id = metadata.id();
        let update_time = metadata.update_time();

        // For fuse_count calculating
        let mut in_range = true;

        // If id should be ignored
        if let Some(ignore_id_lt) = self.strategy.ignore_id_lt {
            if id < ignore_id_lt {
                tracing::trace!("{id} < {ignore_id_lt}");
                in_range = false;
            }
        }

        // If send time is older than limit
        if let Some(ignore_time_lt) = self.strategy.ignore_time_lt {
            if update_time < ignore_time_lt {
                tracing::trace!("{update_time} < {ignore_time_lt}");
                in_range = false;
            }
        }

        // If there is no fuse limit, we will not stop until out of range
        if self.strategy.fuse_limit.is_none() {
            return in_range;
        }

        if found.is_same() {
            // If articles are the same, just add fuse count.
            self.fuse_count += 1
        } else if in_range {
            // Only reset the count if in range
            // When out of range, no fuse_count reset is performed
            self.fuse_count = 0;
        } else {
            // But if not in range anymore, add fuse count too!
            self.fuse_count += 1;
        }

        let result = self.fuse_count <= self.strategy.fuse_limit.unwrap_or(0);
        tracing::debug!(
            "id: {}/{:?}, fuse: {}/{:?}, is_same: {}",
            id,
            self.strategy.ignore_id_lt,
            self.fuse_count,
            self.strategy.fuse_limit,
            found.is_same(),
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
#[derive(Debug, Clone)]
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
