use async_trait::async_trait;
use futures::{stream::BoxStream, TryStreamExt, StreamExt};

use reqwest::{Response, Url};
use serde::{Deserialize, Serialize};

use crate::{
    resource::{
        cartoon::{CartoonPage, Thumbnail, ThumbnailList, PagerTop, PagerDetail},
        information::{Announce, InformationPage, AjaxAnnounceList}, post::PostSource,
    },
    Error, Page,
};

use super::resource::ResourceClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiServer {
    pub id: String,
    pub url: Url,
    pub name: String,
}

pub struct ApiClient {
    pub client: reqwest::Client,
    pub api_server: ApiServer,
}

impl ApiClient {
    async fn information_get(&self, href: &str) -> Result<Response, Error> {
        let url = self.api_server.url.join(href)?;
        self.client.get(url).send().await.map_err(Error::from)
    }

        fn information_href(&self, announce_id: i32) -> String {
        format!(
            "information/detail/{announce_id}/1/10/1",
            announce_id = announce_id
        )
    }

    fn ajax_href(&self, offset: i32) -> String {
        format!("information/ajax_announce?offset={offset}", offset = offset)
    }

    pub async fn information(
        &self,
        announce_id: i32,
    ) -> Result<(InformationPage, kuchiki::NodeRef), Error> {
        let href = self.information_href(announce_id);
        let html = self.information_get(&href).await?.text().await?;

        InformationPage::from_html(html)
    }

    pub async fn ajax_announce_list(&self, offset: i32) -> Result<AjaxAnnounceList, Error> {
        let href = self.ajax_href(offset);
        self.information_get(&href)
            .await?
            .json::<AjaxAnnounceList>()
            .await
            .map_err(Error::from)
    }

    pub async fn announce_list(&self, offset: i32) -> Result<Vec<Announce>, Error> {
        let ajax_announce = self.ajax_announce_list(offset).await?;
        let ajax_announce_list = ajax_announce.announce_list;
        let announce_iter = ajax_announce_list.into_iter().map(Announce::from);
        let announce_list = announce_iter.collect();
        Ok(announce_list)
    }

    pub fn announce_stream(&self) -> BoxStream<Announce> {
        let stream = futures::stream::unfold((0, self), next_ajax);

        let stream = stream.flat_map(|ajax_announce| {
            let list = ajax_announce.announce_list;
            let iter = list.into_iter().map(Announce::from);
            futures::stream::iter(iter)
        });

        Box::pin(stream)
    }

    pub fn announce_try_stream(
        &self,
    ) -> BoxStream<Result<Announce, Error>> {
        let stream = futures::stream::try_unfold((0, self), try_next_ajax);
        let stream = stream
            .map_ok(|ajax_announce| {
                ajax_announce
                    .announce_list
                    .into_iter()
                    .map(Announce::from)
                    .map(Ok)
            })
            .map_ok(futures::stream::iter)
            .try_flatten();

        Box::pin(stream)
    }
}

async fn next_ajax(
    (index, client): (i32, &ApiClient),
) -> Option<(AjaxAnnounceList, (i32, &ApiClient))> {
    if index < 0 {
        return None;
    }

    let announce = client.ajax_announce_list(index).await.ok()?;
    let length = if announce.is_over_next_offset {
        -1
    } else {
        announce.length
    };

    Some((announce, (length, client)))
}

async fn try_next_ajax(
    (index, client): (i32, &ApiClient),
) -> Result<Option<(AjaxAnnounceList, (i32, &ApiClient))>, Error> {
    if index < 0 {
        return Ok(None);
    }

    let announce = client.ajax_announce_list(index).await;
    let announce = match announce {
        Ok(announce) => announce,
        Err(error) => return Err(error),
    };
    let length = if announce.is_over_next_offset {
        -1
    } else {
        announce.length
    };

    Ok(Some((announce, (length, client))))
}

impl ApiClient {
    async fn cartoon_get(&self, href: &str) -> Result<Response, Error> {
        let url = self.api_server.url.join(href)?;
        self.client.get(url).send().await.map_err(Error::from)
    }

    fn cartoon_thumbnail_href(num: i32) -> String {
        format!("cartoon/thumbnail_list/{num}", num = num)
    }

    fn cartoon_pager_top_href(current_page_id: i32, page_set: i32) -> String {
        format!("cartoon/pager/0/{current_page_id}/{page_set}")
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

    pub async fn thumbnail_list(&self, page: i32) -> Result<ThumbnailList, Error> {
        let href = Self::cartoon_thumbnail_href(page);
        let result: ThumbnailList = self.cartoon_get(&href).await?.json().await?;

        Ok(result)
    }

    pub async fn cartoon_pager_top(
        &self,
        current_page_id: i32,
        page_set: i32,
    ) -> Result<PagerTop, Error> {
        let href = Self::cartoon_pager_top_href(current_page_id, page_set);
        let result = self.cartoon_get(&href).await?.json().await?;

        Ok(result)
    }

    pub async fn cartoon_pager_detail(
        &self,
        current_page_id: i32,
        page_set: i32,
    ) -> Result<PagerDetail, Error> {
        let href = Self::cartoon_pager_detail_href(current_page_id, page_set);
        let result = self.cartoon_get(&href).await?.json().await?;

        Ok(result)
    }

    pub async fn cartoon(&self, id: i32) -> Result<CartoonPage, Error> {
        let href = Self::cartoon_detail_href(id);
        let html = self.cartoon_get(&href).await?.text().await?;

        CartoonPage::from_html(html).map(|(cartoon, _)| cartoon)
    }

    pub fn thumbnail_stream(&self) -> BoxStream<Thumbnail> {
        let stream = futures::stream::unfold((0, self), next_thumbnails);
        let stream = stream.flat_map(futures::stream::iter);

        Box::pin(stream)
    }

    pub fn thumbnail_try_stream(&self) -> BoxStream<Result<Thumbnail, Error>> {
        let stream = futures::stream::try_unfold((0, self), try_next_thumbnails);
        let stream = stream
            .map_ok(|x| x.into_iter().map(Ok))
            .map_ok(futures::stream::iter)
            .try_flatten();

        Box::pin(stream)
    }
}

async fn next_thumbnails(
    (page, client): (i32, &ApiClient),
) -> Option<(Vec<Thumbnail>, (i32, &ApiClient))> {
    let list = client.thumbnail_list(page).await.ok()?;
    list.0.map(|thumbnails| (thumbnails, (page + 1, client)))
}

async fn try_next_thumbnails(
    (page, client): (i32, &ApiClient),
) -> Result<Option<(Vec<Thumbnail>, (i32, &ApiClient))>, Error> {
    let list = client.thumbnail_list(page).await?;
    let result = list.0.map(|thumbnails| (thumbnails, (page + 1, client)));

    Ok(result)
}

#[async_trait]
impl ResourceClient<Announce> for ApiClient {
    type P = (InformationPage, kuchiki::NodeRef);
    fn try_stream(&self) -> BoxStream<Result<Announce, Error>> {
        self.announce_try_stream()
    }
    async fn page(&self, resource: &Announce) -> Result<Self::P, Error> {
        self.information(resource.announce_id).await
    }
}

impl PostSource<Announce> for ApiClient {
    fn post_source(&self) -> crate::resource::post::sources::Source {
        crate::resource::post::sources::Source::Announce(self.api_server.id.clone())
    }
}

#[async_trait]
impl ResourceClient<Thumbnail> for ApiClient {
    type P = CartoonPage;
    fn try_stream(&self) -> BoxStream<Result<Thumbnail, Error>> {
        self.thumbnail_try_stream()
    }
    async fn page(&self, resource: &Thumbnail) -> Result<Self::P, Error> {
        self.cartoon(resource.id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        service::resource::{ResourceService},
        FetchStrategy,
    };

    use super::*;
    use futures::{stream, StreamExt, TryStreamExt};

    #[tokio::test]
    async fn test_try_stream_and_then() {
        let stream = stream::iter(vec![Ok(1), Ok(2), Err(3)]);
        let stream = stream.and_then(|x| async move { Ok(x + 1) });
        let vec = stream.collect::<Vec<_>>().await;
        println!("{:?}", vec);
    }

    #[tokio::test]
    async fn test_latest_rua() -> Result<(), Box<dyn std::error::Error>> {
        let collection = crate::database::test::init_db()
            .await?
            .collection::<Announce>("announce");

        let client = ApiClient {
            client: reqwest::Client::builder()
                .user_agent(crate::client::ua())
                .build()?,
            api_server: ApiServer {
                id: "PROD1".to_string(),
                url: reqwest::Url::parse("https://api-pc.so-net.tw/")?,
                name: "美食殿堂".to_string(),
            },
        };

        let strategy = FetchStrategy {
            fuse_limit: 5,
            ignore_id_lt: 1852,
        };

        let service = ResourceService::new(client, strategy, collection);

        let mut announces = service.latests().await?;
        println!("{:?}", announces);
        for mut announce in announces.iter_mut().rev().take(5) {
            announce.title.title = "So-net 不會用的標題".to_string();
        }
        for announce in announces.iter() {
            service.collection.upsert(announce).await?;
        }

        let (page, _node) = service.page(&announces[0]).await?;
        tracing::info!("{:?}", page);

        Ok(())
    }

    //     #[tokio::test]
    //     async fn test_latest_announces() -> Result<(), Box<dyn std::error::Error>> {
    //         init_trace();

    //         // let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    //         let client = ApiClient {
    //             client: reqwest::Client::builder()
    //                 .user_agent(crate::client::ua())
    //                 .build()?,
    //             api_server: ApiServer {
    //                 id: "PROD1".to_string(),
    //                 url: reqwest::Url::parse("https://api-pc.so-net.tw/")?,
    //                 name: "美食殿堂".to_string(),
    //             },
    //         };

    //         let service = ResourceService {
    //             client,
    //             strategy: FetchStrategy {
    //                 fuse_limit: 5,
    //                 ignore_id_lt: 1852,
    //             },
    //             collection: ResourceCollection::new(
    //                 crate::database::test::init_db()
    //                     .await?
    //                     .collection("announce"),
    //             ),
    //         };

    //         tracing::info!("User-Agent: {}", crate::client::ua());

    //         // tokio::spawn(async move {
    //         //     while let Some(announce) = rx.recv().await {
    //         //         tracing::info!("{:?}", announce);
    //         //     }
    //         // });

    //         let mut announces = service.latests().await?;
    //         println!("{:?}", announces);
    //         for mut announce in announces.iter_mut().rev().take(5) {
    //             announce.title.title = "So-net 不會用的標題".to_string();
    //         }
    //         for announce in announces.iter() {
    //             service.collection.upsert(announce).await?;
    //         }

    //         let (page, _node) = service.page(&announces[0]).await?;
    //         tracing::info!("{:?}", page);

    //         service
    //             .sync(|announce| {
    //                 async move {
    //                     tracing::debug!("{:?}", announce);
    //                     Ok(())
    //                 }
    //                 .boxed()
    //             })
    //             .await?;

    //         Ok(())
    //     }
}
