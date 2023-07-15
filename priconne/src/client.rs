//! Client module
//!
//! This module defines [`ResourceClient`] which is responsible for fetching and
//! parsing resources. The client provide a [`try_stream`] method which returns
//! a [`Stream`] of [`ResourceMetadata`].
//!
//! All clients should inherit from [`ResourceClient`], which provides
//! a unified interface for fetching and parsing resources. Clients with a
//! [`Sendable`] response are [`SendableResourceClient`].
//!
//! By specifying a collection and a [`FetchStrategy`], we can make a client
//! [`MemorizedResourceClient`], which will cache the resources in the database.
//! Only new resources will be fetched and parsed.

use async_trait::async_trait;
use futures::{future, stream::BoxStream, StreamExt, TryStreamExt};
use mongodb::{bson::doc, Collection};

use crate::{
    database::ResourceMetadataCollection, message::Sendable, resource::ResourceMetadata, Error,
};

/// Types for [`MemorizedResourceClient`].
mod memorize;
pub use memorize::{FetchState, FetchStrategy, MemorizedResourceClient, MetadataFindResult};

/// The response from a resource client.
pub trait ResourceResponse {
    fn telegraph_content(&self, _extra: Option<String>) -> Result<Option<String>, crate::Error> {
        Ok(None)
    }
}

/// `ResourceClient` is a client fetching and parsing resources.
///
/// Such a client is only responsible for fetching and parsing resources.
#[async_trait]
pub trait ResourceClient<M>
where
    Self: Sync + Send,
    M: ResourceMetadata,
{
    type Response: ResourceResponse;
    fn try_stream(&self) -> BoxStream<Result<M, Error>>;
    async fn get_by_id(&self, id: i32) -> Result<Self::Response, Error>;
    async fn fetch(&self, resource: &M) -> Result<Self::Response, Error> {
        self.get_by_id(resource.id()).await
    }
    /// Create a [`MemorizedResourceClient`] from this client.
    fn memorize(
        self,
        collection: Collection<M>,
        strategy: FetchStrategy,
    ) -> MemorizedResourceClient<M, Self>
    where
        Self: Sized,
    {
        MemorizedResourceClient::new(self, strategy, collection)
    }
}

/// When the response of a resource client is sendable, the client is sendable
pub trait SendableResourceClient<M> = ResourceClient<M>
where
    <Self as ResourceClient<M>>::Response: Sendable,
    M: ResourceMetadata;
