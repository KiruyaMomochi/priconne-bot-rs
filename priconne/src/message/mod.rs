use crate::utils::SplitPrefix;
use linked_hash_set::LinkedHashSet;

use crate::{
    resource::{
        information::{Announce, InformationPage},
        post::{sources::Source, NewMessage},
    },
    RegexTagger,
};

#[derive(Debug)]
pub struct MessageBuilder {
    // pub page: &'a InformationPage,
    // pub announce_id: i32,
    // pub telegraph_page: &'a telegraph_rs::Page,
    pub tagger: RegexTagger,
}

// impl<'a> InformationMessageBuilder<'a> {
//     pub fn new(
//         page: &'a InformationPage,
//         announce_id: i32,
//         telegraph_page: &'a telegraph_rs::Page,
//         tagger: &'a RegexTagger,
//     ) -> Self {
//         Self {
//             page,
//             announce_id,
//             telegraph_page,
//             tagger,
//         }
//     }
// }

fn insert_tags_announce(tags: &mut LinkedHashSet<String>, information: &InformationPage) {
    if let Some(icon) = information.icon {
        tags.insert_if_absent(icon.to_tag().to_string());
    }
    for tag in crate::extract_tag(&information.title) {
        tags.insert_if_absent(tag);
    }
}

impl MessageBuilder {
    pub fn build_message_announce(
        &self,
        announce: &Announce,
        information: &InformationPage,
        api_id: String,
        telegraph: &telegraph_rs::Page,
    ) -> NewMessage {
        // let (title, tags) = tags(&information, &self.tagger);
        let _link = &telegraph.url;
        let _id = announce.announce_id;
        let time = information
            .date
            .map_or(announce.replace_time, |d| d.with_timezone(&chrono::Utc));
        // let time = information.date.map_or("Unknown".to_string(), |date| {
        //     date.format(utils::api_date_format::FORMAT).to_string()
        // });
        let mut tags = LinkedHashSet::new();
        // * original tags should have been inserted.
        let display_title = self.tagger.tag_title(&information.title, &mut tags);

        NewMessage {
            title: information.title.clone(),
            display_title: display_title.to_string(),
            source: Source::Announce(api_id),
            id: announce.announce_id,
            create_time: time,
            history: None,
            tags,
            events: information.events.clone(),
            telegraph: Some(telegraph.url.to_string()),
            silent: false,
        }
    }

    fn build_message(
        &self,
        title: &str,
        tags: &LinkedHashSet<String>,
        message_sender: &NewMessage,
    ) -> String {
        // let (title, tags) = tags(&page, &self.tagger);
        let link = &message_sender.telegraph.as_ref().unwrap();
        let id = message_sender.id;
        let time = message_sender.create_time;
        let events = &message_sender.events;

        let mut tag_str = String::new();

        for tag in tags {
            tag_str.push('#');
            tag_str.push_str(tag);
            tag_str.push(' ');
        }

        if !tag_str.is_empty() {
            tag_str.pop();
            tag_str.push('\n');
        }

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
            event_str.insert(0, '\n');
            event_str.push('\n');
        }

        let head = format!("{tag}<b>{title}</b>\n", tag = tag_str, title = title);

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
