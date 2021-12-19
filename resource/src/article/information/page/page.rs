use chrono::{DateTime, FixedOffset};
use kuchiki::{ElementData, NodeDataRef, NodeRef};

use super::Icon;
use priconne_core::{Error, Page};

#[derive(Debug)]
pub struct InformationPage {
    pub title: String,
    pub icon: Option<Icon>,
    pub date: Option<DateTime<FixedOffset>>,
    pub content: kuchiki::NodeRef,
}

#[derive(Debug)]
pub struct InformationPageNoContent {
    pub title: String,
    pub icon: Option<Icon>,
    pub date: Option<DateTime<FixedOffset>>,
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
        utils::trim_leading_whitespace(messages);
        let content = messages_node.as_node().clone();

        if title_node.text_contents().is_empty() {
            return Err(Error::EmptyTitleError);
        }

        Ok(InformationPage {
            title: get_title(&title_node)?,
            date: get_date(&date_node),
            icon: get_icon(&date_node),
            content,
        })
    }
}

impl InformationPage {
    pub fn split(self) -> (InformationPageNoContent, kuchiki::NodeRef) {
        (
            InformationPageNoContent {
                date: self.date,
                icon: self.icon,
                title: self.title,
            },
            self.content,
        )
    }
}

fn get_title(title_node: &NodeDataRef<ElementData>) -> Result<String, Error> {
    let title_node = title_node
        .as_node()
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
        .find(|x| x.starts_with("icon_"))
        .map_or(None, Icon::from_classname)
}

fn get_date(date_node: &NodeDataRef<ElementData>) -> Option<DateTime<FixedOffset>> {
    let date_text_node = &date_node.as_node().first_child()?;
    let date_text = date_text_node.as_text()?.borrow();

    utils::string_to_date(&date_text.trim(), "%Y/%m/%d %H:%M").ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use kuchiki::traits::TendrilSink;
    use std::path::Path;
    use utils::HOUR;

    #[test]
    fn test_information_page_from_document() {
        let path = Path::new("tests/information_page.html");
        let document = kuchiki::parse_html().from_utf8().from_file(path).unwrap();
        let page = InformationPage::from_document(document).unwrap();
        assert_eq!(page.title, "【活動】臺灣藝術家聯合會藝術研習班");
        assert_eq!(
            page.date,
            Some(
                FixedOffset::east(8 * HOUR)
                    .ymd(2021, 10, 19)
                    .and_hms(11, 55, 0)
            )
        );
        assert_eq!(page.icon, Some(Icon::Activity));
    }

    #[tokio::test]
    async fn test_information_page_from_document_div() {
        let path = Path::new("tests/information_div.html");
        let document = kuchiki::parse_html().from_utf8().from_file(path).unwrap();
        let page = InformationPage::from_document(document).unwrap();

        utils::insert_br_after_div(&page.content);
        let content = telegraph_rs::doms_to_nodes(page.content.children().clone()).unwrap();
        println!("{:?}", content);
        let json = serde_json::to_string(&content).unwrap();
        let telegraph = telegraph_rs::Telegraph::new("test")
            .create()
            .await
            .unwrap()
            .create_page(&page.title, &json, false)
            .await
            .unwrap();
        println!("{}", telegraph.url);
    }
}
