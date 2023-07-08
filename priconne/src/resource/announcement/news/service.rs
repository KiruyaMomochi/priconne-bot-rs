use async_trait::async_trait;
use futures::{stream::BoxStream, Stream, TryStreamExt};

use html5ever::tendril::TendrilSink;
use reqwest::{Response, Url};

use crate::{
    client::ResourceClient,
    resource::{
        announcement::{sources::AnnouncementSource, AnnouncementResponse},
        news::{News, NewsList, NewsPage},
        service::AnnouncementClient,
    },
    Error, Page,
};

#[derive(Debug, Clone)]
pub struct NewsClient {
    pub client: reqwest::Client,
    pub server: Url,
}

impl NewsClient {
    fn server(&self) -> &url::Url {
        &self.server
    }

    fn url(&self, href: &str) -> Result<url::Url, Error> {
        self.server().join(href).map_err(Error::from)
    }

    async fn get_raw(&self, href: &str) -> Result<Response, Error> {
        let url = self.url(href)?;
        self.client.get(url).send().await.map_err(Error::from)
    }

    fn list_href(&self, page: i32) -> String {
        format!("news?page={page}")
    }

    fn href(&self, news_id: i32) -> String {
        format!("news/newsDetail/{news_id}")
    }

    async fn get(&self, news_id: i32) -> Result<AnnouncementResponse<NewsPage>, Error> {
        let href = self.href(news_id);
        let response = self.get_raw(&href).await?;
        let url = response.url().clone();
        let html = response.text().await?;

        let response = AnnouncementResponse {
            post_id: news_id,
            source: AnnouncementSource::Website,
            url,
            page: NewsPage::from_html(html)?,
        };

        Ok(response)
    }

    async fn list(&self, page: i32) -> Result<NewsList, Error> {
        let href = self.list_href(page);
        let html = self.get_raw(&href).await?.text().await?;

        NewsList::from_html(html)
    }

    fn try_stream(&self) -> impl Stream<Item = Result<News, Error>> + '_ {
        let stream =
            futures::stream::try_unfold((Some(self.list_href(1)), self), try_next_news_list);

        stream
            .map_ok(|news_list| news_list.news_list.into_iter().map(Ok))
            .map_ok(futures::stream::iter)
            .try_flatten()
    }
}

#[async_trait]
impl ResourceClient<News> for NewsClient {
    type Response = AnnouncementResponse<NewsPage>;
    fn try_stream(&self) -> BoxStream<Result<News, Error>> {
        Box::pin(self.try_stream())
    }
    async fn get_by_id(&self, id: i32) -> Result<Self::Response, Error> {
        self.get(id).await
    }

    // fn url_by_id(&self, id: i32) -> Result<url::Url, Error> {
    //     self.url(&self.href(id))
    // }
}

impl AnnouncementClient<News> for NewsClient {
    type Page = NewsPage;

    fn source(&self) -> AnnouncementSource {
        AnnouncementSource::Website
    }
}

async fn try_next_news_list(
    (href, client): (Option<String>, &NewsClient),
) -> Result<Option<(NewsList, (Option<String>, &NewsClient))>, Error> {
    let href = match href {
        Some(href) => href,
        None => return Ok(None),
    };

    let response = client.get_raw(&href).await?;
    let text = response.text().await?;
    let document = kuchikiki::parse_html().one(text);
    let news_list = NewsList::from_document(document)?;
    let next_href = news_list.next_href.clone();

    Ok(Some((news_list, (next_href, client))))
}

#[cfg(test)]
mod tests {
    use crate::client::FetchStrategy;
    use reqwest::Url;

    use super::*;

    #[tokio::test]
    async fn test_latest_news() -> Result<(), Box<dyn std::error::Error>> {
        let collection = crate::database::tests::init_db().await?.collection("news");
        let client = NewsClient {
            client: reqwest::Client::new(),
            server: Url::parse("http://www.princessconnect.so-net.tw")?,
        };
        let strategy = FetchStrategy {
            fuse_limit: Some(5),
            ignore_id_lt: Some(9999),
            ..Default::default()
        };
        client.source();
        let client = client.memorize(collection, strategy);

        let news = client.latests().await?;
        println!("{news:?}");

        Ok(())
    }
}
