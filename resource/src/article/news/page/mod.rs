mod list;
mod news;
mod page;

pub use list::NewsList;
pub use news::News;
pub use page::NewsPage;

use kuchiki::NodeRef;
use priconne_core::Error;
use utils::HOUR;

pub fn get_date(date_node: NodeRef) -> Result<chrono::Date<chrono::FixedOffset>, Error> {
    let date_text = date_node.into_text_ref().ok_or(Error::KuchikiError)?;
    let date_text = date_text.borrow();
    let date_text = date_text.trim();

    let offset = chrono::offset::FixedOffset::east(8 * HOUR);
    let date = chrono::NaiveDate::parse_from_str(date_text, "%Y.%m.%d")?;
    let date = chrono::Date::<chrono::offset::FixedOffset>::from_utc(date, offset);

    Ok(date)
}

pub fn get_category(category_node: NodeRef) -> Result<Option<String>, Error> {
    let category_node = category_node
        .into_element_ref()
        .ok_or(Error::KuchikiError)?;
    let category_attributes = category_node.attributes.borrow();
    let category_class = category_attributes
        .get("class")
        .ok_or(Error::KuchikiError)?
        .trim();
    let category = if category_class.starts_with("ac") {
        Some(
            category_node
                .as_node()
                .first_child()
                .ok_or(Error::KuchikiError)?
                .into_text_ref()
                .ok_or(Error::KuchikiError)?
                .borrow()
                .to_owned(),
        )
    } else {
        None
    };

    Ok(category)
}
