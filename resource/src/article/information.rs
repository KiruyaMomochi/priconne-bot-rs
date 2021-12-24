mod bot;
mod database;
mod page;

use database::*;
use page::*;

use async_trait::async_trait;
use futures::StreamExt;
use priconne_core::{Client, Error, Page};
use reqwest::Response;

#[async_trait]
pub trait InformationClient: Sync {
    async fn information_get(&self, href: &str) -> Result<Response, Error>;

    fn information_href(&self, announce_id: i32) -> String {
        format!(
            "information/detail/{announce_id}/1/10/1",
            announce_id = announce_id
        )
    }

    fn ajax_href(&self, offset: i32) -> String {
        format!("information/ajax_announce?offset={offset}", offset = offset)
    }

    async fn information_page(&self, announce_id: i32) -> Result<(InformationPage, kuchiki::NodeRef), Error> {
        let href = self.information_href(announce_id);
        let html = self.information_get(&href).await?.text().await?;

        InformationPage::from_html(html)
    }

    async fn ajax_announce(&self, offset: i32) -> Result<AjaxAnnounceList, Error> {
        let href = self.ajax_href(offset);
        self.information_get(&href)
            .await?
            .json::<AjaxAnnounceList>()
            .await
            .map_err(Error::from)
    }

    async fn announce_list(&self, offset: i32) -> Result<Vec<Announce>, Error> {
        let ajax_announce = self.ajax_announce(offset).await?;
        let ajax_announce_list = ajax_announce.announce_list;
        let announce_iter = ajax_announce_list.into_iter().map(Announce::from);
        let announce_list = announce_iter.collect();
        Ok(announce_list)
    }
}

#[async_trait::async_trait]
impl InformationClient for Client {
    async fn information_get(&self, href: &str) -> Result<Response, Error> {
        let url = self.api_server().join(href)?;
        self.get(url).send().await.map_err(Error::from)
    }
}

impl<T: ?Sized> InformationExt for T where T: InformationClient + Clone + Send {}
pub trait InformationExt: InformationClient + Clone + Send {
    fn information_stream(&self) -> Box<dyn futures::Stream<Item = Announce> + '_ + Send> {
        let stream = futures::stream::unfold((0, self.clone()), next_ajax);

        let stream = stream.flat_map(|ajax_announce| {
            let list = ajax_announce.announce_list;
            let iter = list.into_iter().map(|announce| Announce::from(announce));
            futures::stream::iter(iter)
        });

        Box::new(stream)
    }
}

async fn next_ajax<T: InformationExt>(
    (index, client): (i32, T),
) -> Option<(AjaxAnnounceList, (i32, T))> {
    if index < 0 {
        return None;
    }

    let announce = client.ajax_announce(index).await.ok()?;
    let length = if announce.is_over_next_offset {
        -1
    } else {
        announce.length
    };

    Some((announce, (length, client)))
}
