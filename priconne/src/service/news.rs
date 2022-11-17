use async_trait::async_trait;
use futures::{stream::BoxStream, TryStreamExt, StreamExt};

use html5ever::tendril::TendrilSink;
use reqwest::{Response, Url};

use crate::{
    resource::news::{News, NewsPage, NewsList},
    service::resource::ResourceClient,
    Error, Page,
};

pub struct NewsClient {
    pub client: reqwest::Client,
    pub news_server: Url,
}

impl NewsClient {
    fn news_server(&self) -> &url::Url {
        &self.news_server
    }

    async fn news_get(&self, href: &str) -> Result<Response, Error> {
        let url = self.news_url(href)?;
        self.client.get(url).send().await.map_err(Error::from)
    }
    
    fn news_url(&self, href: &str) -> Result<url::Url, Error> {
        self.news_server().join(href).map_err(Error::from)
    }

    fn news_list_href(&self, page: i32) -> String {
        format!("news?page={page}", page = page)
    }
    fn news_detail_href(&self, news_id: i32) -> String {
        format!("news/newsDetail/{news_id}", news_id = news_id)
    }

    async fn news_page(&self, news_id: i32) -> Result<(NewsPage, kuchiki::NodeRef), Error> {
        let href = self.news_detail_href(news_id);
        let html = self.news_get(&href).await?.text().await?;

        NewsPage::from_html(html)
    }

    async fn news_page_from_href(&self, href: &str) -> Result<(NewsPage, kuchiki::NodeRef), Error> {
        let html = self.news_get(href).await?.text().await?;

        NewsPage::from_html(html)
    }

    async fn news_list_page(&self, page: i32) -> Result<NewsList, Error> {
        let href = self.news_list_href(page);
        let html = self.news_get(&href).await?.text().await?;

        NewsList::from_html(html).map(|(news_list, _)| news_list)
    }

    async fn news_list(&self, page: i32) -> Result<Vec<News>, Error> {
        Ok(self.news_list_page(page).await?.news_list)
    }

    fn news_stream(&self) -> BoxStream<News> {
        let stream = futures::stream::unfold((Some(self.news_list_href(1)), self), next_news_list);

        let stream =
            stream.flat_map(|news_list| futures::stream::iter(news_list.news_list.into_iter()));

        Box::pin(stream)
    }

    fn news_try_stream(&self) -> BoxStream<Result<News, Error>> {
        let stream =
            futures::stream::try_unfold((Some(self.news_list_href(1)), self), try_next_news_list);

        let stream = stream
            .map_ok(|news_list| news_list.news_list.into_iter().map(Ok))
            .map_ok(futures::stream::iter)
            .try_flatten();

        Box::pin(stream)
    }
}

async fn next_news_list(
    (href, client): (Option<String>, &NewsClient),
) -> Option<(NewsList, (Option<String>, &NewsClient))> {
    let href = href?;
    let response = client.news_get(&href).await.ok()?;
    let text = response.text().await.ok()?;
    let document = kuchiki::parse_html().one(text);
    let news_list = NewsList::from_document(document).ok()?.0;
    let next_href = news_list.next_href.clone();

    Some((news_list, (next_href, client)))
}

async fn try_next_news_list(
    (href, client): (Option<String>, &NewsClient),
) -> Result<Option<(NewsList, (Option<String>, &NewsClient))>, Error> {
    let href = match href {
        Some(href) => href,
        None => return Ok(None),
    };

    let response = client.news_get(&href).await?;
    let text = response.text().await?;
    let document = kuchiki::parse_html().one(text);
    let news_list = NewsList::from_document(document)?.0;
    let next_href = news_list.next_href.clone();

    Ok(Some((news_list, (next_href, client))))
}

#[async_trait]
impl ResourceClient<News> for NewsClient {
    type P = (NewsPage, kuchiki::NodeRef);
    fn try_stream(&self) -> BoxStream<Result<News, Error>> {
        self.news_try_stream()
    }
    async fn page(&self, resource: &News) -> Result<Self::P, Error> {
        self.news_page(resource.id).await
    }
}

#[cfg(test)]
mod tests {
    
    use crate::{news::NewsClient, service::resource::{ResourceService}, FetchStrategy};
    use reqwest::Url;

    #[tokio::test]
    async fn test_latest_news() -> Result<(), Box<dyn std::error::Error>> {
        let collection = crate::database::test::init_db().await?.collection("news");
        let client = NewsClient {
            client: reqwest::Client::new(),
            news_server: Url::parse("http://www.princessconnect.so-net.tw")?,
        };
        let strategy = FetchStrategy {
            fuse_limit: 5,
            ignore_id_lt: 9999,
        };
        let service = ResourceService::new(client, strategy, collection);
        
        let news = service.latests().await?;
        println!("{:?}", news);

        Ok(())
    }
}
