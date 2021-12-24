use chrono::{FixedOffset, DateTime, TimeZone};
use kuchiki::{NodeDataRef, ElementData, traits::NodeIterator};
use utils::HOUR;

#[derive(Debug)]
pub struct EventPeriod {
    pub start: DateTime<FixedOffset>,
    pub end: DateTime<FixedOffset>,
    pub name: String,
}

fn parse_period(period_str: &str) -> Option<(DateTime<FixedOffset>, DateTime<FixedOffset>)> {
    let offset = FixedOffset::east(8 * HOUR);
    // 2021/12/27 05:00
    let fmt = "%Y/%m/%d %H:%M";

    let period_str = period_str.trim();
    let (start, end) = period_str.split_once(&['～', '~'][..])?;
    let start = start.trim();
    let end = end.trim();

    let start = offset.datetime_from_str(start, fmt).ok()?;
    let end = offset.datetime_from_str(end, fmt).ok()?;

    Some((start, end))
}

pub fn get_events(content_node: &NodeDataRef<ElementData>) -> Vec<EventPeriod> {
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
        let name = name.trim_start_matches("■");

        if let Some((start, end)) = parse_period(time) {
            periods.push(EventPeriod {
                start,
                end,
                name: name.to_string(),
            });
        }
    }

    periods
}
