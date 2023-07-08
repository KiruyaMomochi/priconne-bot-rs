//! Database wrappers

use mongodb::{
    bson::doc,
    options::{FindOneAndReplaceOptions, FindOneOptions},
    Collection,
};

use crate::{
    resource::{announcement::sources::AnnouncementSource, Announcement, ResourceMetadata},
    utils::map_title,
};

pub struct AnnouncementCollection(pub Collection<Announcement>);

impl AnnouncementCollection {
    pub fn posts(&self) -> Collection<Announcement> {
        self.0.clone()
    }

    pub async fn find_resource<R>(
        &self,
        resource: &R,
        source: &AnnouncementSource,
    ) -> Result<Option<Announcement>, mongodb::error::Error>
    where
        R: ResourceMetadata,
    {
        self.find(resource.title(), resource.id(), source).await
    }

    /// Find a post that has the same source id, or
    /// * with a similar title, and
    /// * is posted in 24 hours
    async fn find(
        &self,
        title: &str,
        id: i32,
        source: &AnnouncementSource,
    ) -> Result<Option<Announcement>, mongodb::error::Error> {
        let mapped = map_title(title);
        let in24hours = chrono::Utc::now() - chrono::Duration::hours(24);
        // let source_field = &format!("source.{}", source.name());
        let filter = doc! {
            "$or": [
                {
                    "mapped_title": mapped,
                    "data.source": {
                        "$ne": source
                    },
                    "update_time": {
                        "$gte": in24hours
                    },
                },
                {
                    "data.source": source,
                    "data.id": id,
                }
            ]
        };
        tracing::trace!("{filter:?}");

        self.posts()
            .find_one(
                filter,
                FindOneOptions::builder()
                    .sort(doc! {"update_time": -1})
                    .build(),
            )
            .await
    }

    pub async fn upsert(
        &self,
        post: &Announcement,
    ) -> Result<mongodb::results::InsertOneResult, mongodb::error::Error> {
        self.posts().insert_one(post, None).await
    }
}

pub struct ResourceMetadataCollection<R: ResourceMetadata>(Collection<R>);

impl<R> ResourceMetadataCollection<R>
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

#[cfg(test)]
pub mod tests {

    pub async fn init_db() -> Result<mongodb::Database, mongodb::error::Error> {
        let client =
            mongodb::Client::with_uri_str("mongodb://root:example@localhost:27017").await?;
        let db = client.database("test_only_delete_me");
        db.drop(None).await.map(|()| db)
    }
}
