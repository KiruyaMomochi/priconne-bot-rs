use super::{get_category, get_date};
use chrono::{Date, FixedOffset};
use kuchiki::{ElementData, NodeDataRef, NodeRef};
use priconne_core::{Error, Page};
use utils::trim_leading_whitespace;

#[derive(Debug)]
pub struct NewsPage {
    pub date: Date<FixedOffset>,
    pub category: Option<String>,
    pub title: String,
    pub content: kuchiki::NodeRef,
}

#[derive(Debug)]
pub struct NewsPageNoContent {
    pub date: Date<FixedOffset>,
    pub category: Option<String>,
    pub title: String,
}

impl Page for NewsPage {
    fn from_document(document: NodeRef) -> Result<Self, Error> {
        let news_con_node = document
            .select_first(".news_con")
            .map_err(|_| Error::KuchikiError)?;
        let news_con_node = news_con_node.as_node();

        let mut h2_nodes = news_con_node
            .select_first("h2")
            .map_err(|_| Error::KuchikiError)?
            .as_node()
            .children();
        let date_node = h2_nodes.next().ok_or(Error::KuchikiError)?;
        let date = get_date(date_node)?;
        let category_node = h2_nodes.next().ok_or(Error::KuchikiError)?;
        let category = get_category(category_node)?;

        let h3_node = news_con_node
            .select_first("h3")
            .map_err(|_| Error::KuchikiError)?
            .as_node()
            .first_child()
            .ok_or(Error::KuchikiError)?;
        let title = h3_node
            .as_text()
            .ok_or(Error::KuchikiError)?
            .borrow()
            .to_owned();

        let section_node = news_con_node
            .select_first("section")
            .map_err(|_| Error::KuchikiError)?;
        let content = get_content(&section_node)?.clone();

        let news = Self {
            category,
            content,
            date,
            title,
        };

        Ok(news)
    }
}

impl NewsPage {
    pub fn split(self) -> (NewsPageNoContent, kuchiki::NodeRef) {
        (
            NewsPageNoContent {
                category: self.category,
                date: self.date,
                title: self.title,
            },
            self.content,
        )
    }
}

fn get_content(section_node: &NodeDataRef<ElementData>) -> Result<&NodeRef, Error> {
    let section_node = section_node.as_node();
    trim_leading_whitespace(section_node.children());

    let first_child = section_node.first_child().ok_or(Error::KuchikiError)?;
    if &first_child
        .as_element()
        .ok_or(Error::KuchikiError)?
        .name
        .local
        == "h4"
    {
        first_child.detach();
        trim_leading_whitespace(section_node.children());
    }

    Ok(section_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use kuchiki::traits::TendrilSink;
    use utils::HOUR;
    use std::path::Path;

    #[test]
    fn test_from_document() {
        let path = Path::new("tests/news_page.html");
        let document = kuchiki::parse_html().from_utf8().from_file(path).unwrap();
        let page = NewsPage::from_document(document).unwrap();
        assert_eq!(page.date, FixedOffset::east(HOUR).ymd(2021, 08, 24));
        assert_eq!(page.category, Some("活動".to_owned()));
        assert_eq!(page.title, "【轉蛋】《精選轉蛋》新角色「克蘿依（聖學祭）」登場！機率UP活動舉辦預告！".to_owned());
    }
}