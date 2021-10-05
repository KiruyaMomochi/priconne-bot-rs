mod bot;
mod database;
mod page;

use database::*;
use page::*;

use async_trait::async_trait;
use futures::StreamExt;
use kuchiki::traits::TendrilSink;
use priconne_core::{Client, Error, Page};
use reqwest::Response;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{Message, ParseMode},
};

impl<T: ?Sized> NewsExt for T where T: NewsClient + Clone + Send {}

pub trait NewsExt: NewsClient + Clone + Send {
    fn news_stream(&self) -> Box<dyn futures::Stream<Item = News> + '_ + Send> {
        let stream =
            futures::stream::unfold((Some(self.news_list_href(1)), self.clone()), next_news_list);

        let stream =
            stream.flat_map(|news_list| futures::stream::iter(news_list.news_list.into_iter()));

        Box::new(stream)
    }
}

async fn next_news_list<T: NewsExt>(
    (href, client): (Option<String>, T),
) -> Option<(NewsList, (Option<String>, T))> {
    let href = href?;
    let response = client.news_get(&href).await.ok()?;
    let text = response.text().await.ok()?;
    let document = kuchiki::parse_html().one(text);
    let news_list = NewsList::from_document(document).ok()?;
    let next_href = news_list.next_href.clone();

    Some((news_list, (next_href, client)))
}

#[async_trait]
pub trait NewsClient: Sync {
    async fn news_get(&self, href: &str) -> Result<Response, Error>;

    fn news_list_href(&self, page: i32) -> String {
        format!("news?page={page}", page = page)
    }
    fn news_detail_href(&self, news_id: i32) -> String {
        format!("news/newsDetail/{news_id}", news_id = news_id)
    }

    async fn news_page(&self, news_id: i32) -> Result<NewsPage, Error> {
        let href = self.news_detail_href(news_id);
        let html = self.news_get(&href).await?.text().await?;

        NewsPage::from_html(html)
    }

    async fn news_page_from_href(&self, href: &str) -> Result<NewsPage, Error> {
        let html = self.news_get(&href).await?.text().await?;

        NewsPage::from_html(html)
    }

    async fn news_list_page(&self, page: i32) -> Result<NewsList, Error> {
        let href = self.news_list_href(page);
        let html = self.news_get(&href).await?.text().await?;

        NewsList::from_html(html)
    }

    async fn news_list(&self, page: i32) -> Result<Vec<News>, Error> {
        Ok(self.news_list_page(page).await?.news_list)
    }
}

#[async_trait::async_trait]
impl NewsClient for Client {
    async fn news_get(&self, href: &str) -> Result<Response, Error> {
        let url = self.news_server().join(href)?;
        let response = self.get(url).send().await?;
        Ok(response)
    }
}
