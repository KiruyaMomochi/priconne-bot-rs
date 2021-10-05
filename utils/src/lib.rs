use chrono::{DateTime, FixedOffset, TimeZone};
use kuchiki::NodeData;
use markup5ever::local_name;

/// Number of seconds in an hour
pub const HOUR: i32 = 3600;

/// Trim leading space strings from a slibling node
pub fn trim_leading_whitespace(sliblings: kuchiki::iter::Siblings) -> bool {
    for slibling in sliblings {
        match slibling.data() {
            NodeData::Element(element_data) => match element_data.name.local {
                local_name!("br") => slibling.detach(),
                local_name!("div") => {
                    if trim_leading_whitespace(slibling.children()) {
                        return true;
                    }
                    if slibling.children().next().is_none() {
                        slibling.detach();
                    }
                }
                local_name!("p") => {
                    if trim_leading_whitespace(slibling.children()) {
                        return true;
                    }
                    if slibling.children().next().is_none() {
                        slibling.detach();
                    }
                }
                _ => return true,
            },
            NodeData::Text(text) => {
                let mut value = text.borrow_mut();

                *value = value.trim_start().to_string();

                if value.is_empty() {
                    slibling.detach();
                } else {
                    return true;
                }
            }
            _ => continue,
        }
    }

    false
}

/// Date format from priconne api server
pub mod api_date_format {
    use super::string_to_date;
    use chrono::{DateTime, FixedOffset};
    use serde::{Deserialize, Deserializer, Serializer};

    pub const FORMAT: &str = "%Y-%m-%d %H:%M";

    pub fn serialize<S>(date: &DateTime<FixedOffset>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        string_to_date(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

/// Convert string to UTC+8 DateTime
pub fn string_to_date(
    string: &str,
    format: &str,
) -> Result<DateTime<FixedOffset>, chrono::ParseError> {
    let offset: chrono::FixedOffset = chrono::FixedOffset::east(8 * HOUR);

    let datetime = offset.datetime_from_str(string, format)?;
    return Ok(datetime);
}

pub trait SplitOnce {
    /// Split a string by a separator and return the first part and the rest
    /// If the separator is not found, return none
    fn split_once_temp<'a>(&'a self, pattern: char) -> Option<(&'a str, &'a str)>;
}

impl SplitOnce for str {
    /// Split a string by a separator and return the first part and the rest
    /// If the separator is not found, return none
    ///
    /// # Example
    /// ```
    /// assert_eq!("a,b,c".split_once(","), Some(("a", "b,c")));
    /// assert_eq!("a,b,c".split_once(":"), None);
    /// ```
    fn split_once_temp<'a>(&'a self, pattern: char) -> Option<(&'a str, &'a str)> {
        let find_result = self.find(pattern);
        find_result.map(|position| (&self[..position], &self[position + pattern.len_utf8()..]))
    }
}

pub trait SplitPrefix: SplitOnce {
    /// Trim the `prefix` of a string,
    /// then split the rest by a separator and return the first part and the rest
    fn split_prefix<'a>(&'a self, prefix: char, pattern: char) -> Option<(&'a str, &'a str)>;
}

impl SplitPrefix for str {
    /// Trim the `prefix` of a string,
    /// then split the rest by a separator and return the first part and the rest
    ///
    /// # Example
    /// ```
    /// assert_eq!("a,b,c".split_prefix("a", ","), Some(("b", "b,c")));
    /// assert_eq!("a,b,c".split_prefix("a", ":"), None);
    /// assert_eq!("a,b,c".split_prefix("b", ","), None);
    /// ```
    fn split_prefix<'a>(&'a self, prefix: char, pattern: char) -> Option<(&'a str, &'a str)> {
        if self.starts_with(prefix) {
            self[prefix.len_utf8()..].split_once_temp(pattern)
        } else {
            return None;
        }
    }
}

pub mod chrono_date_utc8_as_bson_datetime {
    use crate::HOUR;
    use chrono::{FixedOffset, Utc};
    use mongodb::bson::DateTime;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::result::Result;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<chrono::Date<FixedOffset>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let datetime = DateTime::deserialize(deserializer)?;
        let timezone = chrono::FixedOffset::east(8 * HOUR);
        let datetime = datetime.to_chrono().with_timezone(&timezone);
        Ok(datetime.date())
    }

    /// Serializes a [`chrono::Date`] as a [`bson::DateTime`].
    pub fn serialize<S: Serializer>(
        val: &chrono::Date<FixedOffset>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let datetime = val.and_hms(0, 0, 0);
        let datetime = DateTime::from_chrono(datetime.with_timezone(&Utc));
        datetime.serialize(serializer)
    }
}

pub mod serde_as_string {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::str::FromStr;

    pub fn deserialize<'de, D, T: FromStr>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        let number = string.parse::<T>();
        number.map_err(|_| serde::de::Error::custom("failed to parse string"))
    }

    pub fn serialize<S, T: ToString>(val: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string = val.to_string();
        string.serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kuchiki::traits::TendrilSink;

    #[test]
    fn test_trim_leading_whitespace() {
        let document = kuchiki::parse_html()
            .one("<body><div></div><h1>Test</h1></body>")
            .select_first("body")
            .unwrap();
        let document = document.as_node();
        assert_eq!(trim_leading_whitespace(document.children()), true);
        assert_eq!(document.to_string(), "<body><h1>Test</h1></body>");
    }

    #[test]
    fn test_split_once() {
        let s = "abcdefg";
        let (a, b) = s.split_once_temp('d').unwrap();
        assert_eq!(a, "abc");
        assert_eq!(b, "efg");
    }

    #[test]
    fn test_split_prefix() {
        let s = "abcdefg";
        let (a, b) = s.split_prefix('a', 'd').unwrap();
        assert_eq!(a, "bc");
        assert_eq!(b, "efg");
    }
}
