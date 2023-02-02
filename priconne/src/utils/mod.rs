use chrono::{DateTime, FixedOffset, TimeZone};
use serde::{
    de::{self, Visitor},
    Deserializer,
};
use std::{fmt, str::FromStr};

mod html;
pub use html::*;

/// Number of seconds in an hour
pub const HOUR: i32 = 3600;

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
    Ok(datetime)
}

pub trait SplitPrefix {
    /// Trim the `prefix` of a string,
    /// then split the rest by a separator and return the first part and the rest
    fn split_prefix(&self, prefix: char, pattern: char) -> Option<(&'_ str, &'_ str)>;
}

impl SplitPrefix for str {
    /// Trim the `prefix` of a string,
    /// then split the rest by a separator and return the first part and the rest
    ///
    /// # Example
    /// ```
    /// use crate::utils::SplitPrefix;
    /// assert_eq!("a,b,c".split_prefix('a', ','), Some(("", "b,c")));
    /// assert_eq!("a,b,c".split_prefix('a', ':'), None);
    /// assert_eq!("a,b,c".split_prefix('b', ','), None);
    /// ```
    fn split_prefix(&self, prefix: char, pattern: char) -> Option<(&'_ str, &'_ str)> {
        if self.starts_with(prefix) {
            self[prefix.len_utf8()..].split_once(pattern)
        } else {
            None
        }
    }
}

pub mod chrono_date_utc8_as_bson_datetime {
    use crate::utils::HOUR;
    
    use mongodb::bson::DateTime;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::result::Result;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<chrono::NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let datetime = DateTime::deserialize(deserializer)?;
        let timezone = chrono::FixedOffset::east_opt(8 * HOUR).unwrap();
        let datetime = datetime.to_chrono().with_timezone(&timezone);
        Ok(datetime.date_naive())
    }

    /// Serializes a [`chrono::Date`] as a [`bson::DateTime`].
    pub fn serialize<S: Serializer>(
        val: &chrono::NaiveDate,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let datetime = val.and_hms_opt(0, 0, 0).unwrap();
        let timezone = chrono::FixedOffset::east_opt(8 * HOUR).unwrap();
        let datetime = DateTime::from_chrono(datetime.and_local_timezone(timezone).unwrap());
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

pub fn string_or_i32<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOri32;

    impl<'de> Visitor<'de> for StringOri32 {
        type Value = i32;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or i32")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            FromStr::from_str(value).map_err(de::Error::custom)
        }

        fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v)
        }
    }

    deserializer.deserialize_any(StringOri32)
}

pub fn replace_relative_path(
    url: &url::Url,
    nodes: &mut Vec<telegraph_rs::Node>,
) -> Result<(), url::ParseError> {
    for node in nodes {
        if let telegraph_rs::Node::NodeElement(telegraph_rs::NodeElement {
            tag: _,
            attrs: Some(attrs),
            children: _,
        }) = node
        {
            if let Some(src) = attrs.get_mut("src") {
                if src.starts_with("./") {
                    *src = url.join(src)?.to_string();
                }
                if src.starts_with('/') && url.has_host() {
                    *src = url.origin().unicode_serialization() + src;
                }
            }
        }

        if let telegraph_rs::Node::NodeElement(telegraph_rs::NodeElement {
            tag: _,
            attrs: _,
            children: Some(children),
        }) = node
        {
            replace_relative_path(url, children)?;
        }
    }
    Ok(())
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
    fn test_split_prefix() {
        let s = "abcdefg";
        let (a, b) = s.split_prefix('a', 'd').unwrap();
        assert_eq!(a, "bc");
        assert_eq!(b, "efg");
    }
}
