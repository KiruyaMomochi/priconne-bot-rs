use async_trait::async_trait;
use futures::{
    future::{self},
    stream::BoxStream,
    StreamExt, TryStreamExt,
};
use mongodb::{bson::doc, options::FindOneAndReplaceOptions, Collection};

use serde::{de::DeserializeOwned, Serialize};

use crate::{Error, resource::Resource};

use super::{FetchStrategy, update::ResourceFindResult};

/// `ResourceClient` is a client fetching and parsing resources.
#[async_trait]
pub trait ResourceClient<R>
where
    R: Resource<IdType = i32> + Sync,
{
    type Response;
    fn try_stream(&self) -> BoxStream<Result<R, Error>>;
    async fn get_by_id(&self, id: R::IdType) -> Result<Self::Response, Error>;
    async fn page(&self, resource: &R) -> Result<Self::Response, Error> {
        self.get_by_id(resource.id()).await
    }
    // fn url_by_id(&self, id: R::IdType) -> Result<url::Url, Error>;
    // fn url(&self, resource: &R) -> Result<url::Url, Error> {
    //     self.url_by_id(resource.id())
    // }
}
pub struct ResourceService<R, Client>
where
    Client: ResourceClient<R>,
    R: Resource<IdType = i32> + Sync,
{
    pub client: Client,
    pub strategy: FetchStrategy,
    pub collection: ResourceCollection<R>,
}

#[async_trait]
impl<R, Client> ResourceClient<R> for ResourceService<R, Client>
where
    Client: ResourceClient<R> + Sync + Send,
    R: Resource<IdType = i32> + Serialize + DeserializeOwned + Unpin + Send + Sync,
{
    type Response = Client::Response;
    fn try_stream(&self) -> BoxStream<Result<R, Error>> {
        self.client.try_stream()
    }
    async fn get_by_id(&self, id: i32) -> Result<Self::Response, Error> {
        self.client.get_by_id(id).await
    }
    // fn url_by_id(&self, id: i32) -> Result<url::Url, Error> {
    //     self.client.url_by_id(id)
    // }
}

impl<R, Client> ResourceService<R, Client>
where
    Client: ResourceClient<R> + Sync + Send,
    R: Resource<IdType = i32> + Serialize + DeserializeOwned + Unpin + Send + Sync,
{
    async fn updated(&self, item: R) -> Result<ResourceFindResult<R>, Error> {
        let in_db = self.collection.find(&item).await?;

        let update = match in_db {
            Some(in_db) => {
                ResourceFindResult::from_found(item, in_db)
            }
            None => ResourceFindResult::from_new(item),
        };

        Ok(update)
    }

    async fn upsert(&self, item: &R) -> Result<Option<R>, Error> {
        self.collection.upsert(item).await.map_err(Error::from)
    }

    /// Fetches all resources and their state in the database, as a stream.
    fn compared_stream<'stream>(&'stream self) -> BoxStream<Result<ResourceFindResult<R>, Error>>
    where
        Self: Sync,
        R: 'stream,
    {
        let result = self
            .try_stream()
            .and_then(|item| self.updated(item))
            .into_stream();

        Box::pin(result)
    }

    /// Fetches resources that are new or updated in the database, as a stream.
    pub fn fused_stream<'stream>(&'stream self) -> BoxStream<Result<ResourceFindResult<R>, Error>>
    where
        Self: Sync,
        R: Send + 'stream,
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
                future::ok(fetch_state.keep_going(update.item().id(), update.is_update()))
            })
            .try_filter(|update| future::ready(update.is_not_same()));

        Box::pin(result)
    }

    /// Fetches resources that are new or updated in the database, as a vector.
    pub async fn latests(&self) -> Result<Vec<ResourceFindResult<R>>, Error>
    where
        R: Send,
    {
        let stream = self.fused_stream();
        let result: Vec<_> = stream.try_collect().await?;

        let result = result.into_iter().rev().collect();

        Ok(result)
    }
}

impl<R, Client> ResourceService<R, Client>
where
    Client: ResourceClient<R> + Sync + Send,
    R: Resource<IdType = i32> + Serialize + DeserializeOwned + Unpin + Send + Sync,
{
    pub fn new(client: Client, strategy: FetchStrategy, collection: Collection<R>) -> Self {
        Self {
            client,
            strategy,
            collection: ResourceCollection::new(collection),
        }
    }
}
pub struct ResourceCollection<R: Resource>(Collection<R>);

impl<R> ResourceCollection<R>
where
    R: Resource<IdType = i32> + Serialize + DeserializeOwned + Unpin + Send + Sync,
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

    pub async fn find_by_id(&self, id: &R::IdType) -> Result<Option<R>, mongodb::error::Error> {
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
