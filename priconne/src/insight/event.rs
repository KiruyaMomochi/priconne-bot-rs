use crate::utils::HOUR;
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use kuchikiki::{traits::NodeIterator, ElementData, NodeDataRef};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct EventInAnnouncement {
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub start: DateTime<Utc>,
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub end: DateTime<Utc>,
    pub title: String,
}

fn parse_period(period_str: &str) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    let offset = FixedOffset::east_opt(8 * HOUR).unwrap();
    // 2021/12/27 05:00
    let fmt = "%Y/%m/%d %H:%M";

    let period_str = period_str.trim();
    let (start, end) = period_str.split_once(&['～', '~'][..])?;
    let start = start.trim();
    let end = end.trim();

    let start = offset
        .datetime_from_str(start, fmt)
        .ok()?
        .with_timezone(&Utc);
    let end = offset.datetime_from_str(end, fmt).ok()?.with_timezone(&Utc);

    Some((start, end))
}

pub fn get_events(content_node: &NodeDataRef<ElementData>) -> Vec<EventInAnnouncement> {
    let mut periods = Vec::new();

    let iter = content_node.as_node().descendants().text_nodes();
    let iter = iter.clone().zip(iter.skip(1));
    for (name, time) in iter {
        let name = name.borrow();
        let name = name.trim();
        let time = time.borrow();
        let time = time.trim();

        if !name.ends_with("期間") {
            continue;
        }
        let name = name.trim_start_matches('■');

        if let Some((start, end)) = parse_period(time) {
            periods.push(EventInAnnouncement {
                start,
                end,
                title: name.to_string(),
            });
        }
    }

    periods
}
