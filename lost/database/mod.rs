
#[cfg(test)]
pub mod test {
    use chrono::TimeZone;
    use linked_hash_set::LinkedHashSet;


    use super::*;

    async fn add_post(collection: &PostCollection) -> Result<Vec<Post>, mongodb::error::Error> {
        let now_news = Post::new(
            "新測試新聞".to_string(),
            LinkedHashSet::new(),
            chrono::Utc::now(),
            PostSources::new_news(2),
            Vec::new(),
            "https://example.com".to_string(),
        );

        let old_news = Post::new(
            "舊測試新聞".to_string(),
            LinkedHashSet::new(),
            chrono::Utc.timestamp(612921600, 0),
            PostSources::new_news(2),
            Vec::new(),
            "https://example.com".to_string(),
        );

        collection
            .posts()
            .insert_many([&now_news, &old_news], None)
            .await?;
        Ok(vec![now_news, old_news])
    }

    pub async fn init_db() -> Result<mongodb::Database, mongodb::error::Error> {
        let client =
            mongodb::Client::with_uri_str("mongodb://root:example@localhost:27017").await?;
        let db = client.database("test_only_delete_me");
        db.drop(None).await.map(|()| db)
    }

    pub fn init_trace() {
        let subscriber = tracing_subscriber::fmt()
            // .with_max_level(tracing::Level::TRACE)
            .with_env_filter("priconne=trace")
            .finish();
        tracing::subscriber::set_global_default(subscriber).unwrap();
        tracing_log::LogTracer::init().unwrap();
    }

    pub async fn init_post_collection() -> Result<PostCollection, mongodb::error::Error> {
        let db = init_db().await?;
        let collection = PostCollection(db.collection("posts"));
        Ok(collection)
    }

    #[tokio::test]
    async fn find() -> Result<(), mongodb::error::Error> {
        let collection = init_post_collection().await?;
        add_post(&collection).await?;

        let (fake_id, old_id, new_id) = (0, 1, 2);
        let fake_announce_source = Source::Announce("X".to_string());
        let announce_source = Source::Announce("A".to_string());
        let news_source = Source::News;

        // let fake_announce = sources::AnnounceSource {
        //     api: "X".to_string(),
        //     id: 0,
        // };
        // let fake_news = sources::NewsSource { id: 0 };
        // let old_announce = sources::AnnounceSource {
        //     api: "A".to_string(),
        //     id: 1,
        // };
        // let new_announce = sources::AnnounceSource {
        //     api: "A".to_string(),
        //     id: 2,
        // };
        // let old_news = sources::NewsSource { id: 1 };
        // let new_news = sources::NewsSource { id: 2 };

        // println!("{}", Source::from(new_news.clone()).bson());

        macro_rules! assert_some {
            ($title:literal, $id:expr, $source:ident) => {
                let find_result = collection
                    .find($title, $id, &$source.clone().into())
                    .await?;
                assert!(find_result.is_some());
            };
        }
        macro_rules! assert_none {
            ($title:literal, $id:expr, $source:ident) => {
                let find_result = collection
                    .find($title, $id, &$source.clone().into())
                    .await?;
                assert!(find_result.is_none());
            };
        }

        assert_some!("新測試新聞", fake_id, fake_announce_source);
        assert_none!("新測試新聞", fake_id, news_source);
        assert_some!("新測試新聞", new_id, announce_source);
        assert_some!("新測試新聞", new_id, news_source);
        assert_some!("新測試新聞", old_id, announce_source);
        assert_none!("新測試新聞", old_id, news_source);

        assert_none!("舊測試新聞", fake_id, fake_announce_source);
        assert_none!("舊測試新聞", fake_id, news_source);
        assert_none!("舊測試新聞", new_id, announce_source);
        assert_some!("舊測試新聞", new_id, news_source);
        assert_none!("舊測試新聞", old_id, announce_source);
        assert_none!("舊測試新聞", old_id, news_source);

        Ok(())
    }
}