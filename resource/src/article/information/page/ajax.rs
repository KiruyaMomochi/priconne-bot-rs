use chrono::{DateTime, FixedOffset, TimeZone};
use serde::{Deserialize, Serialize};
use utils::api_date_format;

#[derive(Serialize, Deserialize, Debug)]
pub struct AjaxAnnounceList {
    pub announce_list: Vec<AjaxAnnounce>,
    pub per_page: i32,
    pub base_url: String,
    pub total_rows: i32,
    pub offset: i32,
    pub is_over_next_offset: bool,
    pub length: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AjaxAnnounce {
    pub announce_id: i32,
    pub language: i32,
    pub category: i32,
    pub status: i32,
    pub platform: i32,
    pub slider_flag: i32,
    #[serde(with = "api_date_format")]
    pub from_date: DateTime<FixedOffset>,
    #[serde(with = "api_date_format")]
    pub to_date: DateTime<FixedOffset>,
    pub replace_time: i64,
    pub priority: i32,
    pub end_date_slider_image: Option<String>,
    pub title: AnnounceTitle,
    pub link_num: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnnounceTitle {
    pub title: String,
    pub slider_image: Option<String>,
    pub thumbnail_image: Option<String>,
    pub banner_ribbon: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Announce {
    pub announce_id: i32,
    language: i32,
    category: i32,
    status: i32,
    platform: i32,
    slider_flag: i32,
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    from_date: DateTime<chrono::Utc>,
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    to_date: DateTime<chrono::Utc>,
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub replace_time: DateTime<chrono::Utc>,
    priority: i32,
    end_date_slider_image: Option<String>,
    pub title: AnnounceTitle,
    link_num: i32,
}

impl From<AjaxAnnounce> for Announce {
    fn from(announce: AjaxAnnounce) -> Self {
        Self {
            announce_id: announce.announce_id,
            language: announce.language,
            category: announce.category,
            status: announce.status,
            platform: announce.platform,
            slider_flag: announce.slider_flag,
            from_date: announce.from_date.with_timezone(&chrono::offset::Utc),
            to_date: announce.to_date.with_timezone(&chrono::offset::Utc),
            replace_time: chrono::Utc.timestamp_opt(announce.replace_time, 0).unwrap(),
            priority: announce.priority,
            end_date_slider_image: announce.end_date_slider_image,
            title: announce.title,
            link_num: announce.link_num,
        }
    }
}
