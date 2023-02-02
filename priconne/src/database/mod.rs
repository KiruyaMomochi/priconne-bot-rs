use mongodb::{bson::doc, options::FindOneOptions, Collection};

use crate::resource::{
    post::{sources::Source, Post},
    same::map_titie,
    Resource,
};

pub struct PostCollection(Collection<Post>);

impl PostCollection {
    pub fn posts(&self) -> Collection<Post> {
        self.0.clone()
    }

    pub async fn find_resource<R>(
        &self,
        resource: &R,
        source: &Source,
    ) -> Result<Option<Post>, mongodb::error::Error>
    where
        R: Resource<IdType = i32>,
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
        source: &Source,
    ) -> Result<Option<Post>, mongodb::error::Error> {
        // let source: Source = source.into();
        let mapped = map_titie(title);
        let in24hours = chrono::Utc::now() - chrono::Duration::hours(24);
        let source_field = &format!("source.{}", source.name());

        self.posts()
            .find_one(
                doc! {
                    "$or": [
                        {
                            "mapped_title": mapped,
                            source_field: {},
                            "update_time": {
                                "$gte": in24hours
                            }
                        },
                        {
                            source_field: id
                        }
                    ]
                },
                FindOneOptions::builder()
                    .sort(doc! {"update_time": -1})
                    .build(),
            )
            .await
    }

    pub async fn upsert(
        &self,
        post: &Post,
    ) -> Result<mongodb::results::InsertOneResult, mongodb::error::Error> {
        self.posts().insert_one(post, None).await
    }
}

#[cfg(test)]
pub mod test {
    pub async fn init_db() -> Result<mongodb::Database, mongodb::error::Error> {
        let client =
            mongodb::Client::with_uri_str("mongodb://root:example@localhost:27017").await?;
        let db = client.database("test_only_delete_me");
        db.drop(None).await.map(|()| db)
    }
}
