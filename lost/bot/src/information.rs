use futures::StreamExt;
use linked_hash_set::LinkedHashSet;
use log::info;
use priconne_core::{Error, RegexTagger};
use resource::{information::{InformationPage, InformationClient, Announce, InformationExt}, message::{MessageBuilder, Post}};
use std::pin::Pin;
use telegraph_rs::doms_to_nodes;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{ChatId, Message, ParseMode},
};
use crate::utils::SplitPrefix;

use crate::bot::Bot;

#[derive(Debug)]
pub struct InformationMessageBuilder<'a> {
    pub page: &'a InformationPage,
    pub announce_id: i32,
    pub telegraph_page: &'a telegraph_rs::Page,
    pub tagger: &'a RegexTagger,
}

impl<'a> MessageBuilder for InformationMessageBuilder<'a> {
    fn build_message(&self) -> String {
        let (title, tags) = tags(&self.page, &self.tagger);
        let link = &self.telegraph_page.url;
        let id = self.announce_id;
        let time = self.page.date.map_or("Unknown".to_string(), |date| {
            date.format(utils::api_date_format::FORMAT).to_string()
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

        let mut event_str = String::new();

        for event in &self.page.events {
            event_str.push_str("- ");
            event_str.push_str(&event.title);
            event_str.push_str(": \n   ");
            event_str.push_str(event.start.format("%m/%d %H:%M").to_string().as_str());
            event_str.push_str(" - ");
            event_str.push_str(event.end.format("%m/%d %H:%M").to_string().as_str());
            event_str.push_str("\n");
        }
        if !event_str.is_empty() {
            event_str.insert_str(0, "\n");
            event_str.push_str("\n");
        }

        let head = format!(
            "{tag}<b>{title}</b>\n",
            tag = tag_str,
            title = title
        );

        let tail = format!(
            "{link}\n{time} <code>#{id}</code>",
            link = link,
            time = time,
            id = id
        );

        let message = format!("{}{}{}", head, event_str, tail);
        message
    }
}

fn tags<'a>(page: &'a InformationPage, tagger: &'a RegexTagger) -> (&'a str, Vec<String>) {
    let mut title: &str = &page.title;
    let mut tags: LinkedHashSet<String> = LinkedHashSet::new();

    if let Some(icon) = page.icon {
        tags.insert(icon.to_tag().to_string());
    }
    if let Some((category, base_title)) = title.split_prefix('【', '】') {
        title = base_title;
        tags.insert(category.to_string());
    }

    tags.extend(tagger.tag_iter(title));
    (title, tags.into_iter().collect())
}

pub struct SentInformation {
    message: Message,
    telegraph: telegraph_rs::Page,
    page: InformationPage,
}

impl<C: InformationClient + Clone + Send> Bot<C> {
    pub async fn announce_by_id(
        &self,
        announce_id: i32,
        chat_id: ChatId,
    ) -> Result<SentInformation, Error> {
        info!("Announcing by id {}", announce_id);

        let page;
        let content;
        {
            let (p, c) = self.client.information(announce_id).await?;
            page = p;
            utils::insert_br_between_div(&c);

            content = serde_json::to_string(&doms_to_nodes(c.children()))?;
        };
        info!("Got information page {}", page.title);

        let telegraph_page = self
            .telegraph
            .create_page(&page.title, &content, false)
            .await?;
        info!("Published telegraph page {}", telegraph_page.url);

        let post = Post::from_information(&page, "", "0", announce_id);


        let disable_notification = page.title.contains("外掛停權");

        let message = self
            .bot
            .send_message(chat_id, "post.build_html()")
            .parse_mode(ParseMode::Html)
            .disable_notification(disable_notification)
            .await?;

        Ok(SentInformation {
            message,
            page,
            telegraph: telegraph_page,
        })
    }

    pub async fn announce(
        &self,
        announce: &Announce,
        chat_id: ChatId,
    ) -> Result<SentInformation, Error> {
        return self.announce_by_id(announce.announce_id, chat_id).await;
    }

    pub async fn announce_all(&self, limit: i32, min: i32, chat_id: ChatId) -> Result<(), Error> {
        log::info!("announce_all with limit {} and min {}", limit, min);

        let stream = self.client.information_stream();
        let mut stream = unsafe { Pin::new_unchecked(stream) };

        let mut skip_counter = 0;
        let mut vec = Vec::new();
        while let Some(announce) = stream.next().await {
            if skip_counter >= limit {
                break;
            }

            let sent_information = self.database.check_sent_announce(&announce).await?;
            if sent_information.is_not_found() {
                log::info!(
                    "hit information {}: {}",
                    announce.announce_id,
                    announce.title.title
                );
                if announce.announce_id >= min {
                    skip_counter = 0;
                }

                vec.push((announce, sent_information));
            } else {
                skip_counter += 1;
                log::info!(
                    "ign information {}: {} ({}/{})",
                    announce.announce_id,
                    announce.title.title,
                    skip_counter,
                    limit
                );
            }
        }

        for (announce, result) in vec.iter().rev() {
            let message = self.announce(announce, chat_id.clone()).await?;
            self.database
                .upsert_information_in_sent(
                    &result,
                    announce,
                    &message.message,
                    &message.telegraph.url,
                )
                .await?;
        }

        Ok(())
    }
}

