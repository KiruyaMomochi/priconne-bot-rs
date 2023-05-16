mod page;
use async_trait::async_trait;
pub use page::*;
use reqwest::Url;

use crate::{
    message::{Message, Sendable},
    service::{
        api::ApiClient,
        resource::{MemorizedResourceClient, ResourceClient, MetadataFindResult},
        PriconneService, ResourceService,
    },
    Error,
};

pub struct CartoonService {
    client: MemorizedResourceClient<Thumbnail, ApiClient>,
}

pub struct Cartoon {
    pub id: i32,
    pub episode: String,
    pub title: String,
    pub image_src: Url,
}

impl Cartoon {
    pub fn caption(&self) -> String {
        format!(
            "<b>第 {episode} 話</b>: {title}\n{image_src} <code>#{id}</code>",
            episode = self.episode,
            title = self.title,
            image_src = self.image_src,
            id = self.id
        )
    }
}

impl Sendable for Cartoon {
    fn message(&self) -> Message {
        Message {
            text: self.caption(),
            silent: false,
            image_src: Some(self.image_src.clone()),
        }
    }
}

#[async_trait]
impl ResourceService<MetadataFindResult<Thumbnail>> for CartoonService {
    async fn collect_latests(
        &self,
        priconne: &PriconneService,
    ) -> Result<Vec<MetadataFindResult<Thumbnail>>, Error> {
        self.client.latests().await
    }
    async fn work(
        &self,
        priconne: &PriconneService,
        result: MetadataFindResult<Thumbnail>,
    ) -> Result<(), Error> {
        let item = result.item();
        let image_src = { self.client.fetch(item).await?.image_src };

        let cartoon = Cartoon {
            id: item.id,
            episode: item.episode.clone(),
            title: item.title.clone(),
            image_src: Url::parse(&image_src)?,
        };

        priconne.chat_manager.send_cartoon(&cartoon).await?;

        Ok(())
    }
}
