mod bot;
mod database;
mod page;

use database::*;
use page::*;

use async_trait::async_trait;
use futures::StreamExt;
use mongodb::Collection;
use priconne_core::{Client, Error, Page};
use reqwest::Response;
use teloxide::prelude::Request;

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

impl<T: ?Sized> CartoonExt for T where T: CartoonClient + Clone + Send {}

pub trait CartoonExt: CartoonClient + Clone + Send {
    fn cartoon_stream(&self) -> Box<dyn futures::Stream<Item = Thumbnail> + '_ + Send> {
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
