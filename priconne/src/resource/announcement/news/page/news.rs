use crate::Error::{self, KuchikiError};
use kuchikiki::NodeRef;
use serde::{Deserialize, Serialize};

use super::{get_category, get_date};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct News {
    #[serde(with = "crate::utils::chrono_date_utc8_as_bson_datetime")]
    pub date: chrono::NaiveDate,
    pub category: Option<String>,
    #[serde(rename = "_id")]
    pub id: i32,
    pub href: String,
    pub title: String,
    pub display_title: String,
}

impl News {
    pub fn from_nodes(dt: &NodeRef, dd: &NodeRef) -> Result<Self, Error> {
        node_to_news(dt, dd)
    }
}

fn node_to_news(dt: &NodeRef, dd: &NodeRef) -> Result<News, Error> {
    let mut dt_nodes = dt.children();

    let date_node = dt_nodes.next().ok_or(KuchikiError)?;
    let date = get_date(date_node)?;

    let category_node = dt_nodes.next().ok_or(KuchikiError)?;
    let category = get_category(category_node)?;

    let mut dd_nodes = dd.children();

    let a_node = dd_nodes.next().ok_or(KuchikiError)?;
    let a_node = a_node.into_element_ref().ok_or(KuchikiError)?;
    let a_attributes = a_node.attributes.borrow();
    let href = a_attributes.get("href").ok_or(KuchikiError)?.to_owned();
    let title = a_attributes.get("title").ok_or(KuchikiError)?.to_owned();
    let display_title = a_node
        .as_node()
        .first_child()
        .ok_or(KuchikiError)?
        .into_text_ref()
        .ok_or(KuchikiError)?
        .borrow()
        .to_owned();

    if !href.starts_with("/news/newsDetail/") {
        return Err(Error::KuchikiError);
    }

    let id_text = &href["/news/newsDetail/".len()..];
    let id = id_text.parse::<i32>()?;

    Ok(News {
        category,
        date,
        id,
        display_title,
        href,
        title,
    })
}
