use crate::{
    client::{MemorizedResourceClient, MetadataFindResult, ResourceClient, ResourceResponse},
    resource::api::ApiClient,
    service::{PriconneService, ResourceService},
    Error,
};

use async_trait::async_trait;
use reqwest::Url;

use super::{Cartoon, Thumbnail};

pub struct CartoonService {
    pub client: MemorizedResourceClient<Thumbnail, ApiClient>,
}

#[async_trait]
impl ResourceService<MetadataFindResult<Thumbnail>> for CartoonService {
    async fn collect_latests(
        &self,
        _priconne: &PriconneService,
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

impl ResourceResponse for crate::resource::cartoon::CartoonPage {}
