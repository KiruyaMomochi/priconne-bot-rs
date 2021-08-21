mod bot;
mod page;

use futures::StreamExt;

use linked_hash_set::LinkedHashSet;
use mongodb::{
    bson::doc,
    options::{FindOneAndReplaceOptions, FindOneOptions},
};
pub use page::*;

use crate::{client::Client, database::{PriconneNewsDatabase, SentMessage}, error::Error, message::{map_titie, MessageBuilder, Tagger}, page::Page, utils::SplitPrefix};
use async_trait::async_trait;
use reqwest::Response;

pub enum InformatonResult {
    Announce(Announce),
    SentAnnounce(SentMessage),
    SentNoAnnounce(SentMessage),
    None,
}

impl InformatonResult {
    pub fn is_found(&self) -> bool {
        match self {
            InformatonResult::Announce(_) => true,
            InformatonResult::SentAnnounce(_) => true,
            InformatonResult::SentNoAnnounce(_) => false,
            InformatonResult::None => false,
        }
    }

    pub fn is_not_found(&self) -> bool {
        return !self.is_found();
    }
}

#[async_trait]
trait InformationDatabase: PriconneNewsDatabase {
    fn announces(&self) -> mongodb::Collection<Announce>;

    async fn check_announce(
        &self,
        announce: &Announce,
    ) -> Result<Option<Announce>, mongodb::error::Error> {
        let found_announce = self.find_announce(announce).await?;
        if let Some(found_announce) = found_announce {
            if found_announce.replace_time == announce.replace_time {
                return Ok(Some(found_announce));
            }
        }
        return Ok(None);
    }

    async fn check_sent_news(
        &self,
        announce: &Announce,
    ) -> Result<InformatonResult, mongodb::error::Error> {
        if let Some(announce) = self.check_announce(announce).await? {
            return Ok(InformatonResult::Announce(announce));
        }

        let found_sent = self.find_sent_information(announce).await?;
        if let Some(found_sent) = found_sent {
            if found_sent.update_time > announce.replace_time {
                self.upsert_announce(announce).await?;
                self.update_sent_information(&found_sent, announce).await?;
                return Ok(InformatonResult::SentAnnounce(found_sent));
            }
            return Ok(InformatonResult::SentNoAnnounce(found_sent));
        }

        Ok(InformatonResult::None)
    }

    async fn find_announce(
        &self,
        announce: &Announce,
    ) -> Result<Option<Announce>, mongodb::error::Error> {
        let collection = self.announces();
        let filter = announce_filter(announce);
        let find_result = collection.find_one(filter.clone(), None).await?;
        Ok(find_result)
    }

    async fn upsert_announce(
        &self,
        announce: &Announce,
    ) -> Result<Option<Announce>, mongodb::error::Error> {
        let collection = self.announces();
        let filter = announce_filter(announce);
        let options = Some(FindOneAndReplaceOptions::builder().upsert(true).build());
        let replace_result = collection
            .find_one_and_replace(filter, announce, options)
            .await?;
        Ok(replace_result)
    }

    async fn find_sent_information(
        &self,
        announce: &Announce,
    ) -> Result<Option<SentMessage>, mongodb::error::Error> {
        let collection = self.sent_messages();
        let filter = sent_filter(announce);
        let find_result = collection
            .find_one(
                filter,
                FindOneOptions::builder()
                    .sort(doc! {"update_time": -1})
                    .build(),
            )
            .await;

        find_result
    }

    async fn update_sent_information(
        &self,
        sent: &SentMessage,
        announce: &Announce,
    ) -> Result<Option<SentMessage>, mongodb::error::Error> {
        let collection = self.sent_messages();

        let find_result = collection
            .find_one_and_update(
                doc! {
                    "message_id": sent.message_id
                },
                doc! {
                    "$currentDate": {
                        "update_time": true,
                    },
                    "$set": {
                        "announce_id": announce.announce_id,
                    },
                },
                None,
            )
            .await;

        find_result
    }

    async fn upsert_sent_information(
        &self,
        sent_information: &InformatonResult,
        announce: &Announce,
        message: &teloxide::types::Message,
        telegraph_url: &str,
    ) -> Result<(), mongodb::error::Error> {
        let collection = self.sent_messages();
        match sent_information {
            InformatonResult::None => {
                let sent_message = SentMessage {
                    announce_id: Some(announce.announce_id),
                    mapped_title: map_titie(&announce.title.title),
                    message_id: message.id,
                    news_id: None,
                    telegraph_url: telegraph_url.to_owned(),
                    update_time: chrono::Utc::now(),
                };
                collection.insert_one(sent_message, None).await?;
            }
            InformatonResult::SentNoAnnounce(found_sent) => {
                collection
                    .find_one_and_update(
                        doc! {
                            "message_id": found_sent.message_id
                        },
                        doc! {
                            "$currentDate": {
                                "update_time": true,
                            },
                            "$set": {
                                "announce_id": announce.announce_id,
                                "message_id": message.id,
                                "telegraph_url": telegraph_url.to_owned(),
                            },
                        },
                        None,
                    )
                    .await?;
            }
            _ => unreachable!()
        };

        Ok(())
    }
}

pub fn announce_filter(announce: &Announce) -> mongodb::bson::Document {
    doc! {
        "announce_id": announce.announce_id,
    }
}

impl InformationDatabase for mongodb::Database {
    fn announces(&self) -> mongodb::Collection<Announce> {
        self.collection("announce")
    }
}

#[derive(Debug)]
pub struct InformationMessageBuilder<'a> {
    pub page: &'a InformationPage,
    pub announce: &'a Announce,
    pub telegraph_page: &'a telegraph_rs::Page,
    pub tagger: &'a Tagger,
}

impl<'a> MessageBuilder for InformationMessageBuilder<'a> {
    fn build_message(&self) -> String {
        let (title, tags) = tags(&self.page, &self.tagger);
        let link = &self.telegraph_page.url;
        let id = self.announce.announce_id;
        let time = self.page.date.map_or("Unknown".to_string(), |date| {
            date.format(crate::utils::api_date_format::FORMAT)
                .to_string()
        });

        let mut tag_str = String::new();

        for tag in &tags {
            tag_str.push_str("#");
            tag_str.push_str(tag);
            tag_str.push_str(" ");
        }

        if !tag_str.is_empty() {
            tag_str.pop();
            tag_str.push('\n');
        }

        let message = format!(
            "{tag}<b>{title}</b>\n{link}\n{time} <code>#{id}</code>",
            tag = tag_str,
            title = title,
            link = link,
            time = time,
            id = id
        );

        message
    }
}

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

    async fn information_page(&self, announce_id: i32) -> Result<InformationPage, Error> {
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

impl<T: ?Sized> InformationExt for T where T: InformationClient + Clone {}
pub trait InformationExt: InformationClient + Clone {
    fn information_stream(&self) -> Box<dyn futures::Stream<Item = Announce> + '_> {
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

pub fn tags<'a>(page: &'a InformationPage, tagger: &'a Tagger) -> (&'a str, Vec<String>) {
    let mut title: &str = &page.title;
    let mut tags: LinkedHashSet<String> = LinkedHashSet::new();

    if let Some(icon) = page.icon {
        tags.insert(icon.to_tag().to_string());
    }
    if let Some((category, base_title)) = title.split_prefix('【', '】') {
        title = base_title;
        tags.insert(category.to_string());
    }

    tags.extend(tagger.tag(title));
    (title, tags.into_iter().collect())
}

pub fn sent_filter(announce: &Announce) -> mongodb::bson::Document {
    let time = chrono::Utc::now() - chrono::Duration::hours(24);
    let mapped_title = &map_titie(&announce.title.title);
    doc! {
        "$or": [
            {
                "mapped_title": mapped_title,
                "announce_id": null,
                "update_time": {
                    "$gte": time
                }
            },
            {
                "mapped_title": mapped_title,
                "announce_id": announce.announce_id
            }
        ]
    }
}
