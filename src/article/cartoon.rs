mod bot;
mod page;

use crate::{client::Client, error::Error, message::MessageBuilder, page::Page};
use async_trait::async_trait;
use futures::StreamExt;
use mongodb::{bson::doc, options::FindOneAndReplaceOptions, Collection};
pub use page::*;
use reqwest::Response;
use teloxide::{
    payloads::SendPhotoSetters,
    prelude::{Request, Requester},
};

#[async_trait]
trait CartoonDatabase {
    fn cartoons(&self) -> mongodb::Collection<Thumbnail>;

    async fn find_cartoon(
        &self,
        thumbnail: &Thumbnail,
    ) -> Result<Option<Thumbnail>, mongodb::error::Error> {
        let collection = self.cartoons();
        let filter = cartoon_filter(thumbnail);
        let find_result = collection.find_one(filter, None).await?;
        Ok(find_result)
    }

    async fn check_cartoon(
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

    async fn upsert_cartoon(
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

fn cartoon_filter(thumbnail: &Thumbnail) -> mongodb::bson::Document {
    doc! {
        "id": thumbnail.id
    }
}

impl CartoonDatabase for mongodb::Database {
    fn cartoons(&self) -> mongodb::Collection<Thumbnail> {
        self.collection("cartoon")
    }
}

#[async_trait]
trait CartoonBot {
    async fn send_cartoon<'a, C>(
        &self,
        chat_id: C,
        image_url: String,
        message: String,
    ) -> Result<teloxide::types::Message, teloxide::RequestError>
    where
        C: Into<teloxide::types::ChatId> + Send;
}

#[async_trait]
impl CartoonBot for teloxide::Bot {
    async fn send_cartoon<'a, C>(
        &self,
        chat_id: C,
        image_url: String,
        message: String,
    ) -> Result<teloxide::types::Message, teloxide::RequestError>
    where
        C: Into<teloxide::types::ChatId> + Send,
    {
        self.send_photo(chat_id, teloxide::types::InputFile::Url(image_url))
            .caption(message)
            .parse_mode(teloxide::types::ParseMode::Html)
            .send()
            .await
    }
}

#[async_trait]
impl CartoonBot for teloxide::adaptors::AutoSend<teloxide::Bot> {
    async fn send_cartoon<'a, C>(
        &self,
        chat_id: C,
        image_url: String,
        message: String,
    ) -> Result<teloxide::types::Message, teloxide::RequestError>
    where
        C: Into<teloxide::types::ChatId> + Send,
    {
        self.inner().send_cartoon(chat_id, image_url, message).await
    }
}

#[async_trait]
pub trait CartoonClient: Sync {
    async fn cartoon_get(&self, href: &str) -> Result<Response, Error>;

    fn cartoon_thumbnail_href(num: i32) -> String {
        format!("cartoon/thumbnail_list/{num}", num = num)
    }

    fn cartoon_pager_top_href(current_page_id: i32, page_set: i32) -> String {
        format!(
            "cartoon/pager/0/{current_page_id}/{page_set}",
            current_page_id = current_page_id,
            page_set = page_set
        )
    }

    fn cartoon_pager_detail_href(current_page_id: i32, page_set: i32) -> String {
        format!(
            "cartoon/pager/1/{current_page_id}/{page_set}",
            current_page_id = current_page_id,
            page_set = page_set
        )
    }

    fn cartoon_detail_href(id: i32) -> String {
        format!("cartoon/detail/{id}", id = id)
    }

    async fn thumbnail_list(&self, page: i32) -> Result<ThumbnailList, Error> {
        let href = Self::cartoon_thumbnail_href(page);
        let result: ThumbnailList = self.cartoon_get(&href).await?.json().await?;

        Ok(result)
    }

    async fn cartoon_pager_top(
        &self,
        current_page_id: i32,
        page_set: i32,
    ) -> Result<PagerTop, Error> {
        let href = Self::cartoon_pager_top_href(current_page_id, page_set);
        let result = self.cartoon_get(&href).await?.json().await?;

        Ok(result)
    }

    async fn cartoon_pager_detail(
        &self,
        current_page_id: i32,
        page_set: i32,
    ) -> Result<PagerDetail, Error> {
        let href = Self::cartoon_pager_detail_href(current_page_id, page_set);
        let result = self.cartoon_get(&href).await?.json().await?;

        Ok(result)
    }

    async fn cartoon_detail(&self, id: i32) -> Result<CartoonPage, Error> {
        let href = Self::cartoon_detail_href(id);
        let html = self.cartoon_get(&href).await?.text().await?;

        CartoonPage::from_html(html)
    }
}

#[async_trait::async_trait]
impl CartoonClient for Client {
    async fn cartoon_get(&self, href: &str) -> Result<Response, Error> {
        let url = self.api_server().join(href)?;
        let response = self.get(url).send().await?;
        Ok(response)
    }
}

impl<T: ?Sized> CartoonExt for T where T: CartoonClient + Clone {}

pub trait CartoonExt: CartoonClient + Clone {
    fn cartoon_stream(&self) -> Box<dyn futures::Stream<Item = Thumbnail> + '_> {
        let stream = futures::stream::unfold((0, self.clone()), next_thumbnails);
        let stream = stream.flat_map(|list| futures::stream::iter(list));

        Box::new(stream)
    }
}

async fn next_thumbnails<T: CartoonExt>(
    (page, client): (i32, T),
) -> Option<(Vec<Thumbnail>, (i32, T))> {
    let list = client.thumbnail_list(page).await.ok()?;
    list.0.map(|thumbnails| (thumbnails, (page + 1, client)))
}

pub struct CartoonMessageBuilder<'a> {
    pub thumbnail: &'a Thumbnail,
    pub page: &'a CartoonPage,
}

impl<'a> MessageBuilder for CartoonMessageBuilder<'a> {
    fn build_message(&self) -> String {
        let episode = &self.thumbnail.episode;
        let title = &self.thumbnail.title;
        let image_url = &self.page.image_src;
        let id = self.thumbnail.id;

        let message = format!(
            "<b>第 {episode} 話</b>: {title}\n{image_url} <code>#{id}</code>",
            episode = episode,
            title = title,
            image_url = image_url,
            id = id
        );

        message
    }
}
