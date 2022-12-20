use chrono::{DateTime, FixedOffset};
use kuchiki::{ElementData, NodeDataRef, NodeRef};
use serde::{Serialize, Deserialize};

use super::Icon;
use crate::{Error, Page, insight::PostPage, resource::post::sources::Source};

#[derive(Debug)]
pub struct InformationPage {
    pub icon: Option<Icon>,
    pub title_node: NodeRef,
    pub date_node: NodeRef,
    pub content_node: NodeRef,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InformationExtra {
    pub icon: Option<Icon>
}

impl Page for InformationPage {
    fn from_document(document: NodeRef) -> Result<Self, Error> {
        let messages_node = document
            .select_first(".messages")
            .map_err(|_| Error::KuchikiError)?;
        let date_node = document
            .select_first(".date")
            .map_err(|_| Error::KuchikiError)?;
        let title_node = document
            .select_first(".title")
            .map_err(|_| Error::KuchikiError)?;

        let messages = messages_node.as_node().children();
        crate::utils::trim_leading_whitespace(messages);
        let content_node = messages_node.as_node().clone();

        if title_node.text_contents().is_empty() {
            return Err(Error::EmptyTitleError);
        }
        
        Ok(Self {
            icon: get_icon(&date_node),
            title_node: title_node.as_node().clone(),
            date_node: date_node.as_node().clone(),
            content_node,
        })
    }
}

impl PostPage for InformationPage {
    type ExtraData = InformationExtra;

    fn content(&self) -> kuchiki::NodeRef {
        self.content_node
    }

    fn create_time(&self) -> Option<DateTime<FixedOffset>> {
        get_date(&self.date_node)
    }

    fn extra(&self) -> Self::ExtraData {
        Self::ExtraData {
            icon: self.icon
        }
    }

    fn tags(&self, tagger: &crate::insight::tagging::RegexTagger) -> linked_hash_set::LinkedHashSet<String> {
        let mut tags = tagger.tag_title(&self.title());
        if let Some(icon) = self.icon {
            tags.insert_if_absent(icon.to_tag().to_string());
        }
        tags
    }

    fn title(&self) -> String {
        get_title(&self.title_node).unwrap_or("".to_owned())
    }
}

fn get_title(title_node: &NodeRef) -> Result<String, Error> {
    let title_node = title_node
        .first_child()
        .ok_or(Error::KuchikiError)?;
    let title_text = title_node.into_text_ref().ok_or(Error::KuchikiError)?;
    let title_text = title_text.borrow().trim().to_owned();
    Ok(title_text)
}

fn get_icon(date_node: &NodeDataRef<ElementData>) -> Option<Icon> {
    let attributes = date_node.attributes.borrow();
    attributes
        .get("class")?
        .split_whitespace()
        .find(|x| x.starts_with("icon_")).and_then(Icon::from_classname)
}

fn get_date(date_node: &NodeRef) -> Option<DateTime<FixedOffset>> {
    let date_text_node = &date_node.first_child()?;
    let date_text = date_text_node.as_text()?.borrow();

    crate::utils::string_to_date(date_text.trim(), "%Y/%m/%d %H:%M").ok()
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use chrono::TimeZone;
//     use kuchiki::traits::TendrilSink;
//     use std::{path::Path, str::FromStr};
//     use crate::utils::HOUR;

//     #[test]
//     fn test_information_page_from_document() {
//         let path = Path::new("tests/information.html");
//         let document = kuchiki::parse_html().from_utf8().from_file(path).unwrap();
//         let page = InformationPage::from_document(document).unwrap();
//         assert_eq!(
//             page.title(),
//             "【活動】特別活動「軍團之戰」舉辦中！(12/18更新)"
//         );
//         assert_eq!(
//             page.date,
//             Some(
//                 FixedOffset::east(8 * HOUR)
//                     .ymd(2021, 12, 17)
//                     .and_hms(11, 55, 0)
//             )
//         );
//         assert_eq!(page.icon, Some(Icon::Special));
//         assert_eq!(page.events.len(), 1);
//     }

//     #[tokio::test]
//     async fn test_information_page_from_document_div() {
//         let path = Path::new("tests/information_div.html");
//         let document = kuchiki::parse_html().from_utf8().from_file(path).unwrap();
//         let page = InformationPage::from_document(document).unwrap();

//         assert_eq!(
//             page.title,
//             "【活動】「12月戰隊競賽」模式變更開始預告！"
//         );
//         assert_eq!(
//             page.date,
//             Some(
//                 FixedOffset::east(8 * HOUR)
//                     .ymd(2021, 12, 19)
//                     .and_hms(11, 55, 0)
//             )
//         );
//         assert_eq!(page.icon, Some(Icon::Activity));
//         assert_eq!(page.events.len(), 3);
//     }
// }
