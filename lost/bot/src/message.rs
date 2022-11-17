use crate::event::EventPeriod;
use chrono::{DateTime, TimeZone};
use linked_hash_set::LinkedHashSet;
use mongodb::bson::{doc, oid};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use crate::utils::SplitPrefix;
pub trait MessageBuilder {
    /// Builds a message
    fn build_message(&self) -> String;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SentMessage {
    pub mapped_title: String,
    pub announce_id: Option<i32>,
    pub news_id: Option<i32>,
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub update_time: DateTime<chrono::Utc>,
    pub telegraph_url: String,
    pub message_id: i32,
}

#[derive(Clone, Debug)]
pub struct Message {
    /// Tags in the message.
    pub tags: LinkedHashSet<String>,
    /// The title in the message.
    pub title: String,
    /// The summary in the message.
    pub summary: Option<String>,
    /// Events in the message.
    pub events: Vec<EventPeriod>,
    /// Telegraph URL.
    pub telegraph: Option<String>,
    /// Created time.
    pub create_time: DateTime<chrono::Utc>,
    /// Source of the message.
    pub sources: Vec<PostSource>,
    /// Silent mode.
    pub silent: bool,
    /// Message ID to reply to.
    pub reply_to: Option<i32>,
}

impl Message {
    pub fn from_post(post: &Post) -> Self {
        Self {
            tags: post.tags.clone(),
            title: post.title.clone(),
            summary: None,
            events: post.events.clone(),
            telegraph: Some(post.telegraph.clone()),
            create_time: post.create_time,
            sources: post.sources.clone(),
            silent: false,
            reply_to: None,
        }
    }

    pub fn build_html(&self) -> String {
        let tag = to_tag_str(self.tags.iter());
        let title = format!("<b>{}</b>", self.title);

        let event = if self.events.is_empty() {
            None
        } else {
            Some(to_event_str(self.events.iter()))
        };

        let mut summary = vec![self.summary.clone(), event]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join("\n\n");
        if !summary.is_empty() {
            summary = format!("\n{}\n", summary)
        };

        let source = to_source_str(self.sources.iter());
        let tail = format!(
            "{link}\n{time} {source}",
            link = self.telegraph.as_ref().map_or("", |x| x),
            time = self.create_time.format("%m/%d %H:%M"),
            source = source
        );

        vec![tag, title, summary, tail]
            .into_iter()
            .filter(|x| !x.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn summary(mut self, summary: String) -> Self {
        self.summary = Some(summary);
        self
    }

    pub fn silent(mut self, silent: bool) -> Self {
        self.silent = silent;
        self
    }

    pub fn reply_to(mut self, reply_to: i32) -> Self {
        self.reply_to = Some(reply_to);
        self
    }
}

fn to_tag_str<T: AsRef<str>>(tags: impl Iterator<Item = T>) -> String {
    let mut string = tags.fold(String::new(), |mut acc, tag| {
        acc.push('#');
        acc.push_str(tag.as_ref());
        acc.push(' ');
        acc
    });

    if !string.is_empty() {
        string.pop();
    }

    string
}

fn to_event_str<'a>(events: impl Iterator<Item = &'a EventPeriod>) -> String {
    let mut event_str = String::new();

    for event in events {
        event_str.push_str("- ");
        event_str.push_str(&event.title);
        event_str.push_str(": \n   ");
        event_str.push_str(event.start.format("%m/%d %H:%M").to_string().as_str());
        event_str.push_str(" - ");
        event_str.push_str(event.end.format("%m/%d %H:%M").to_string().as_str());
        event_str.push('\n');
    }

    if !event_str.is_empty() {
        event_str.pop();
    }

    event_str
}

fn to_source_str<'a>(sources: impl Iterator<Item = &'a PostSource>) -> String {
    let mut source_str = String::new();

    for source in sources {
        match source {
            PostSource::Announce { api, id } => {
                source_str.push_str("<code>");
                source_str.push_str(api);
                source_str.push('#');
                source_str.push_str(&id.to_string());
                source_str.push_str("</code> ");
            }
            PostSource::News { id } => {
                source_str.push_str("<code>");
                source_str.push_str("News");
                source_str.push('#');
                source_str.push_str(&id.to_string());
                source_str.push_str("</code> ");
            }
        }
    }
    if !source_str.is_empty() {
        source_str.pop();
    }

    source_str
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json::json;
    use teloxide::{
        payloads::SendMessageSetters,
        prelude::{Requester, RequesterExt},
        types::{ChatId, Recipient},
    };

    #[tokio::test]
    async fn test_post_deserialize() {
        // let create_date = DateTime::parse_from_rfc3339("2022-07-01T03:55:00+00:00").unwrap().with_timezone(&chrono::Utc);
        // let start_date = DateTime::parse_from_rfc3339("2022-07-02T08:00:00+00:00").unwrap().with_timezone(&chrono::Utc);
        // let end_date = DateTime::parse_from_rfc3339("2022-07-05T07:59:00+00:00").unwrap().with_timezone(&chrono::Utc);
        let create_date = chrono::Utc.ymd(2022, 7, 1).and_hms(3, 55, 0);
        let start_date = chrono::Utc.ymd(2022, 7, 2).and_hms(8, 0, 0);
        let end_date = chrono::Utc.ymd(2022, 7, 5).and_hms(7, 59, 0);

        let bson = doc! {
            "_id": oid::ObjectId::new(),
            "title": "【轉蛋】《公主祭典 獎勵轉蛋》★3「蘭法」期間限定角色登場！舉辦預告！",
            "mapped_title": "《公主祭典 獎勵轉蛋》★3「蘭法」期間限定角色登場！舉辦預告！",
            "region": "TW",
            "sources": [
              {
                "type": "Announce",
                "api": "PROD3",
                "id": 1803
              },
              {
                "type": "News",
                "id": 1774
              }
            ],
            "create_time": create_date,
            "update_time": null,
            "history": null,
            "tags" : [
                "公主祭典",
                "獎勵轉蛋",
                "蘭法",
                "轉蛋",
            ],
            "events": [
              {
                "start": start_date,
                "end": end_date,
                "title": "公主祭典 獎勵轉蛋"
              }
            ],
            "telegraph": "https://telegra.ph/轉蛋公主祭典-獎勵轉蛋3蘭法期間限定角色登場舉辦預告-07-01",
            "message_id": 1800
        };

        let post_from_bson: Post = mongodb::bson::from_document(bson).unwrap();
        let real_post = Post {
            id: post_from_bson.id,
            title: "【轉蛋】《公主祭典 獎勵轉蛋》★3「蘭法」期間限定角色登場！舉辦預告！"
                .to_string(),
            mapped_title: "《公主祭典 獎勵轉蛋》★3「蘭法」期間限定角色登場！舉辦預告！".to_string(),
            region: Region::TW,
            sources: vec![
                PostSource::Announce {
                    api: "PROD3".to_string(),
                    id: 1803,
                },
                PostSource::News { id: 1774 },
            ],
            create_time: create_date,
            update_time: None,
            history: None,
            tags: LinkedHashSet::from_iter(vec![
                "公主祭典".to_string(),
                "蘭法".to_string(),
                "轉蛋".to_string(),
                "獎勵轉蛋".to_string(),
            ]),
            events: vec![EventPeriod {
                start: start_date,
                end: end_date,
                title: "公主祭典 獎勵轉蛋".to_string(),
            }],
            telegraph:
                "https://telegra.ph/轉蛋公主祭典-獎勵轉蛋3蘭法期間限定角色登場舉辦預告-07-01"
                    .to_string(),
            message_id: Some(1800),
        };
        assert_eq!(post_from_bson, real_post);

        let message = Message::from_post(&real_post)
            .silent(true)
            .summary("喵喵喵".to_string())
            .reply_to(48207);

        let mut request = teloxide::Bot::new("5407842045:AAE8essS9PeiQThS-5_Jj7HSfIR_sAcHdKM")
            .auto_send()
            .send_message(
                Recipient::ChannelUsername("@pcrtwstat".to_string()),
                message.build_html(),
            )
            .disable_notification(message.silent);

        if let Some(id) = message.reply_to {
            request = request.reply_to_message_id(id);
        }

        request
            .parse_mode(teloxide::types::ParseMode::Html)
            .await
            .unwrap();
    }
}
