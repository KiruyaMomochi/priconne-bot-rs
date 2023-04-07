use async_trait::async_trait;
use futures::{
    future::{self},
    stream::BoxStream,
    StreamExt, TryStreamExt,
};
use mongodb::{bson::doc, options::FindOneAndReplaceOptions, Collection};

use serde::{de::DeserializeOwned, Serialize};

use crate::{
    insight::AnnouncementPage,
    resource::{announcement::AnnouncementResponse, ResourceMetadata},
    Error,
};

use super::{update::MetadataFindResult, FetchStrategy};

/// `ResourceClient` is a client fetching and parsing resources.
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
}

#[async_trait]
pub trait ResourceService<M>: ResourceClient<M>
where
    M: ResourceMetadata,
{
    async fn latests(&self) -> Result<Vec<MetadataFindResult<M>>, Error>;
}

#[async_trait]
pub trait AnnouncementClient<M, P>:
    ResourceClient<M, Response = AnnouncementResponse<P>>
where
    M: ResourceMetadata,
    P: AnnouncementPage,
{
}

#[async_trait]
pub trait AnnouncementService<M, P>:
    ResourceService<M, Response = AnnouncementResponse<P>>
where
    M: ResourceMetadata,
    P: AnnouncementPage,
{
}

pub struct CommonResourceService<M, Client>
where
    Client: ResourceClient<M>,
    M: ResourceMetadata,
{
    pub client: Client,
    pub strategy: FetchStrategy,
    pub collection: ResourceCollection<M>,
}

#[async_trait]
impl<M, Client> ResourceClient<M> for CommonResourceService<M, Client>
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

impl<M, Client> CommonResourceService<M, Client>
where
    Client: ResourceClient<M>,
    M: ResourceMetadata,
{
    pub fn new(client: Client, strategy: FetchStrategy, collection: Collection<M>) -> Self {
        Self {
            client,
            strategy,
            collection: ResourceCollection::new(collection),
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

    async fn upsert(&self, item: &M) -> Result<Option<M>, Error> {
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
                tracing::trace!(
                    "id = {}, new: {}, update: {}",
                    update.item().id(),
                    update.is_new(),
                    update.is_update()
                );
                future::ok(fetch_state.keep_going(update.item(), update.is_update()))
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

pub struct ResourceCollection<R: ResourceMetadata>(Collection<R>);

impl<R> ResourceCollection<R>
where
    R: ResourceMetadata,
{
    pub fn new(collection: Collection<R>) -> Self {
        Self(collection)
    }

    fn inner(&self) -> &Collection<R> {
        &self.0
    }

    pub async fn find(&self, resource: &R) -> Result<Option<R>, mongodb::error::Error> {
        self.find_by_id(&resource.id()).await
    }

    pub async fn find_by_id(&self, id: &i32) -> Result<Option<R>, mongodb::error::Error> {
        self.inner().find_one(doc! { "_id": id }, None).await
    }

    pub async fn upsert(&self, resource: &R) -> Result<Option<R>, mongodb::error::Error> {
        self.inner()
            .find_one_and_replace(
                doc! { "_id": resource.id() },
                resource,
                FindOneAndReplaceOptions::builder().upsert(true).build(),
            )
            .await
    }
}

pub trait ResourceResponse {
    fn telegraph_content(&self, extra: Option<String>) -> Result<Option<String>, crate::Error> {
        Ok(None)
    }
}

impl ResourceResponse for crate::resource::cartoon::CartoonPage {}
