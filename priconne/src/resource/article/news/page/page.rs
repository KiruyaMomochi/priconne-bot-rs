use super::{get_category, get_date};
use crate::insight::PostPage;
use crate::utils::{trim_leading_whitespace, HOUR};
use crate::{Error, Page};
use chrono::{FixedOffset, NaiveDate};
use kuchiki::{ElementData, NodeDataRef, NodeRef};

#[derive(Debug)]
pub struct NewsPage {
    pub title: String,
    pub category: Option<String>,
    pub date: NaiveDate,
    pub content_node: NodeRef,
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
        let content_node = get_content(&section_node)?.clone();

        Ok(Self {
            category,
            date,
            title,
            content_node,
        })
    }
}

pub struct NewsData {
    pub category: Option<String>,
}

impl PostPage for NewsPage {
    type ExtraData = NewsData;

    fn title(&self) -> String {
        self.title.clone()
    }

    fn content(&self) -> kuchiki::NodeRef {
        self.content_node.clone()
    }

    fn create_time(&self) -> Option<chrono::DateTime<FixedOffset>> {
        let offset = FixedOffset::east(8 * HOUR);
        self.date.and_hms_opt(0, 0, 0)?.and_local_timezone(offset)
    }

    fn extra(&self) -> Self::ExtraData {
        Self::ExtraData {
            category: self.category,
        }
    }
}

fn get_content(section_node: &NodeDataRef<ElementData>) -> Result<&NodeRef, Error> {
    let section_node = section_node.as_node();
    trim_leading_whitespace(section_node.children());

    // So-net put <p> around <div>, which is not correct.
    // This is fixed by parser automatically, so selector like "section > p" does not work.
    // For more information, see:
    // https://stackoverflow.com/questions/8397852/why-cant-the-p-tag-contain-a-div-tag-inside-it/8398003#8398003
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
    use crate::utils::HOUR;
    use chrono::TimeZone;
    use kuchiki::traits::TendrilSink;
    use std::path::Path;

    #[test]
    fn test_from_document() {
        let path = Path::new("tests/news_page.html");
        let document = kuchiki::parse_html().from_utf8().from_file(path).unwrap();
        let (page, _) = NewsPage::from_document(document).unwrap();
        assert_eq!(page.date, FixedOffset::east(HOUR).ymd(2021, 8, 24));
        assert_eq!(page.category, Some("活動".to_owned()));
        assert_eq!(
            page.title,
            "【轉蛋】《精選轉蛋》新角色「克蘿依（聖學祭）」登場！機率UP活動舉辦預告！".to_owned()
        );
        assert_eq!(page.events.len(), 1);
    }

    #[test]
    fn test_from_1376() {
        let path = Path::new("tests/news_1376.html");
        let document = kuchiki::parse_html().from_utf8().from_file(path).unwrap();
        let (page, _) = NewsPage::from_document(document).unwrap();

        assert_eq!(page.date, FixedOffset::east(HOUR).ymd(2021, 10, 26));
        assert_eq!(page.category, Some("活動".to_owned()));
        assert_eq!(
            page.title,
            "【活動】《10月戰隊競賽》限定加碼！特別排名活動".to_owned()
        );
        assert_eq!(page.events.len(), 1);
    }
}
