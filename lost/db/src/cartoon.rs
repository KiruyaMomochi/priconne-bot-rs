use mongodb::{bson::{Document, doc}, Collection, options::FindOneAndReplaceOptions};
use resource::cartoon::Thumbnail;

impl super::Db {
    pub async fn find_cartoon(
        &self,
        thumbnail: &Thumbnail,
    ) -> Result<Option<Thumbnail>, mongodb::error::Error> {
        let collection = self.cartoons();
        let filter = cartoon_filter(thumbnail);
        let find_result = collection.find_one(filter, None).await?;
        Ok(find_result)
    }

    pub async fn check_cartoon(
        &self,
        thumbnail: &Thumbnail,
    ) -> Result<Option<Thumbnail>, mongodb::error::Error> {
        let found_cartoon = self.find_cartoon(thumbnail).await?;

        if let Some(found_cartoon) = found_cartoon {
            if found_cartoon.title == thumbnail.title && found_cartoon.episode == thumbnail.episode
            {
                return Ok(Some(found_cartoon));
            }
        }
        Ok(None)
    }

    pub async fn upsert_cartoon(
        &self,
        thumbnail: &Thumbnail,
    ) -> Result<Option<Thumbnail>, mongodb::error::Error> {
        let collection = self.cartoons();
        upsert_cartoon(&collection, thumbnail).await
    }
}

async fn upsert_cartoon(
    collection: &Collection<Thumbnail>,
    thumbnail: &Thumbnail,
) -> Result<Option<Thumbnail>, mongodb::error::Error> {
    let filter = cartoon_filter(thumbnail);
    let options = Some(FindOneAndReplaceOptions::builder().upsert(true).build());
    let replace_result = collection
        .find_one_and_replace(filter, thumbnail, options)
        .await?;
    Ok(replace_result)
}

fn cartoon_filter(thumbnail: &Thumbnail) -> Document {
    doc! {
        "id": thumbnail.id
    }
}
