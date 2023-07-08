use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PagerTop {
    is_pager: bool,
    latest: i32,
    newer: i32,
    older: i32,
    first: i32,
    pager: Vec<PageTop>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageTop {
    page_id: i32,
    index_text: String,
    current: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PagerDetail {
    is_pager: bool,
    latest: i32,
    newer: i32,
    older: i32,
    first: i32,
    pager: Vec<PageDetail>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageDetail {
    page_id: i32,
    index_text: String,
    current: bool,
    current_page_id: i32,
    page_set: i32,
}
