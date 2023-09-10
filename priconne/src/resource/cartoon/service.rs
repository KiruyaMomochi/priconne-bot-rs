use crate::{
    client::{MemorizedResourceClient, MetadataFindResult, ResourceClient, ResourceResponse},
    resource::api::ApiClient,
    service::{PriconneService, ResourceService},
    Error,
};

use async_trait::async_trait;
use reqwest::Url;

use super::{Cartoon, Thumbnail};

// pub struct CartoonService {
//     pub client: ,
// }

#[async_trait]
impl ResourceService<MetadataFindResult<Thumbnail>>
    for MemorizedResourceClient<Thumbnail, ApiClient>
{
    async fn collect_latests(
        &self,
        _priconne: &PriconneService,
    ) -> Result<Vec<MetadataFindResult<Thumbnail>>, Error> {
        self.latests().await
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
    fn dry_work(&self, metadata: MetadataFindResult<Thumbnail>) {
        tracing::info!("dry_run: work cartoon {}", metadata.item().title)
    }
}

impl ResourceResponse for crate::resource::cartoon::CartoonPage {}
