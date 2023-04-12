mod post;
pub use post::Post;

use mongodb::{bson::doc, options::FindOneOptions, Collection};
use regex::Regex;

use crate::resource::{
    announcement::sources::AnnouncementSource,
    ResourceMetadata,
};

pub struct AnnouncementCollection(pub Collection<Post>);

impl AnnouncementCollection {
    pub fn posts(&self) -> Collection<Post> {
        self.0.clone()
    }

    pub async fn find_resource<R>(
        &self,
        resource: &R,
        source: &AnnouncementSource,
    ) -> Result<Option<Post>, mongodb::error::Error>
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
    ) -> Result<Option<Post>, mongodb::error::Error> {
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
        post: &Post,
    ) -> Result<mongodb::results::InsertOneResult, mongodb::error::Error> {
        self.posts().insert_one(post, None).await
    }
}

/// Create mapped title that not changeed by square bracket or update information.
pub fn map_title(title: &str) -> String {
    let title = title.trim();
    let regex = Regex::new(r#"^\s*(【.+?】)?\s*(.+?)\s*(\(.+更新\))?\s*$"#).unwrap();
    let title = regex.replace(title, "$2");

    title.to_string()
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub async fn init_db() -> Result<mongodb::Database, mongodb::error::Error> {
        let client =
            mongodb::Client::with_uri_str("mongodb://root:example@localhost:27017").await?;
        let db = client.database("test_only_delete_me");
        db.drop(None).await.map(|()| db)
    }

    #[test]
    fn test_map_titie() {
        assert_eq!(
            map_title("「消耗體力時」主角EXP獲得量1.5倍活動！"),
            "「消耗體力時」主角EXP獲得量1.5倍活動！"
        );
        assert_eq!(
            map_title("【活動】【喵喵】「消耗體力時」主角EXP獲得量1.5倍活動！"),
            "【喵喵】「消耗體力時」主角EXP獲得量1.5倍活動！"
        );
        assert_eq!(
            map_title("【活動】「消耗體力時」主角EXP獲得量1.5倍活動！(1/1更新)"),
            "「消耗體力時」主角EXP獲得量1.5倍活動！"
        );
    }
}
