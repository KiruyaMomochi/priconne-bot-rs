use mongodb::{options::{FindOneAndReplaceOptions, FindOneOptions}, bson::{doc, Document}};
use resource::{news::News, message::SentMessage};

impl super::Db {
    pub async fn find_news(&self, news: &News) -> Result<Option<News>, mongodb::error::Error> {
        let collection = self.news();
        let filter = news_filter(news);
        collection.find_one(filter, None).await
    }

    pub async fn check_news(&self, news: &News) -> Result<Option<News>, mongodb::error::Error> {
        if let Some(found_news) = self.find_news(news).await? {
            if found_news.display_title == news.title && found_news.date == news.date {
                return Ok(Some(found_news));
            }
        }

        Ok(None)
    }

    pub async fn upsert_news(&self, news: &News) -> Result<Option<News>, mongodb::error::Error> {
        let collection = self.news();
        let filter = news_filter(news);
        let options = Some(FindOneAndReplaceOptions::builder().upsert(true).build());
        let replace_result = collection
            .find_one_and_replace(filter, news, options)
            .await?;
        Ok(replace_result)
    }

}

pub enum NewsResult {
    News(News),
    SentNews(SentMessage),
    SentNoNews(SentMessage),
    None,
}

impl NewsResult {
    pub fn is_found(&self) -> bool {
        match self {
            NewsResult::News(_) => true,
            NewsResult::SentNews(_) => true,
            NewsResult::SentNoNews(_) => false,
            NewsResult::None => false,
        }
    }

    pub fn is_not_found(&self) -> bool {
        !self.is_found()
    }
}

fn news_filter(news: &News) -> Document {
    doc! {
        "id": news.id,
    }
}
