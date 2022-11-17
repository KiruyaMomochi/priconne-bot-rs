use async_trait::async_trait;
use futures::{
    future::{self, BoxFuture},
    stream::BoxStream,
    StreamExt, TryStreamExt,
};
use mongodb::{bson::doc, options::FindOneAndReplaceOptions, Collection};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    post::PostService,
    resource::{post::PostSource, Resource},
    Error, FetchStrategy,
};

/// `ResourceClient` is a client fetching and parsing resources.
#[async_trait]
pub trait ResourceClient<R>
where
    R: Resource<IdType = i32>,
{
    type P;
    fn try_stream(&self) -> BoxStream<Result<R, Error>>;
    async fn page(&self, resource: &R) -> Result<Self::P, Error>;
}
pub struct ResourceService<R, Client>
where
    Client: ResourceClient<R>,
    R: Resource<IdType = i32>,
{
    pub client: Client,
    pub strategy: FetchStrategy,
    pub collection: ResourceCollection<R>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Update<T> {
    pub item: T,
    pub is_update: bool,
    pub is_new: bool,
}

impl<T> Update<T> {
    pub fn from_new(inner: T) -> Self {
        Self {
            item: inner,
            is_update: true,
            is_new: true,
        }
    }
    pub fn from_found(inner: T, is_update: bool) -> Self {
        Self {
            item: inner,
            is_update,
            is_new: false,
        }
    }
}

#[async_trait]
impl<R, Client> ResourceClient<R> for ResourceService<R, Client>
where
    Client: ResourceClient<R> + Sync + Send,
    R: Resource<IdType = i32> + Serialize + DeserializeOwned + Unpin + Send + Sync,
{
    type P = Client::P;
    fn try_stream(&self) -> BoxStream<Result<R, Error>> {
        self.client.try_stream()
    }
    async fn page(&self, resource: &R) -> Result<Self::P, Error> {
        self.client.page(resource).await
    }
}

impl<R, Client> ResourceService<R, Client>
where
    Client: ResourceClient<R> + Sync + Send,
    R: Resource<IdType = i32> + Serialize + DeserializeOwned + Unpin + Send + Sync,
{
    async fn updated(&self, item: R) -> Result<Update<R>, Error> {
        let in_db = self.collection.find(&item).await?;

        let update = match in_db {
            Some(in_db) => {
                let is_update = Resource::is_update(&item, &in_db);
                Update::from_found(item, is_update)
            }
            None => Update::from_new(item),
        };

        Ok(update)
    }

    async fn upsert(&self, item: &R) -> Result<Option<R>, Error> {
        self.collection.upsert(item).await.map_err(Error::from)
    }

    /// Fetches all resources and their state in the database, as a stream.
    fn compared_stream<'stream>(&'stream self) -> BoxStream<Result<Update<R>, Error>>
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
    fn fused_stream<'stream>(&'stream self) -> BoxStream<Result<R, Error>>
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
                    update.item.id(),
                    update.is_new,
                    update.is_update
                );
                future::ok(fetch_state.keep_going(update.item.id(), update.is_update))
            })
            .try_filter(|update| future::ready(update.is_new || update.is_update))
            .map_ok(|update| update.item);

        Box::pin(result)
    }

    /// Fetches resources that are new or updated in the database, as a vector.
    pub async fn latests(&self) -> Result<Vec<R>, Error>
    where
        R: Send,
    {
        let stream = self.fused_stream();
        let result: Vec<_> = stream.try_collect().await?;

        let result = result.into_iter().rev().collect();

        Ok(result)
    }

    #[allow(dead_code)]
    pub async fn sync<F>(&self, on_resource: F) -> Result<(), Error>
    where
        for<'a> F: Fn(&'a R) -> BoxFuture<'a, Result<(), Error>> + Sync + Send,
        R: Sync + Send,
    {
        let stream = self.fused_stream();
        let result: Vec<_> = stream.try_collect().await?;

        for announce in result.into_iter().rev() {
            on_resource(&announce).await?;
            self.upsert(&announce).await?;
        }

        Ok(())
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

impl<R, Client> PostSource<R> for ResourceService<R, Client>
where
    Client: ResourceClient<R> + PostSource<R>,
    R: Resource<IdType = i32>,
{
    fn post_source(&self) -> crate::resource::post::sources::Source {
        self.client.post_source()
    }
}

impl<R, Client> PostSource<&R> for ResourceService<R, Client>
where
    Client: ResourceClient<R> + PostSource<R>,
    R: Resource<IdType = i32>,
{
    fn post_source(&self) -> crate::resource::post::sources::Source {
        self.client.post_source()
    }
}
