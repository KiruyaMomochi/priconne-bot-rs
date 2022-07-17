use chrono::{DateTime, TimeZone};
use cron::Schedule;
use std::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::Scheduled;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(transparent))]
pub struct Schedules {
    /// List of [`Schedule`]
    #[cfg_attr(feature = "serde", serde(with = "parsing"))]
    schedules: Vec<Schedule>,
}

impl TryFrom<Vec<String>> for Schedules {
    type Error = cron::error::Error;

    fn try_from(vec: Vec<String>) -> Result<Self, Self::Error> {
        let schedules = vec
            .iter()
            .map(|s| Schedule::from_str(s))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { schedules })
    }
}

impl ToString for Schedules {
    fn to_string(&self) -> String {
        self.schedules
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Schedules {
    /// Create a new schedules with the given list of schedules
    pub fn new(schedules: Vec<Schedule>) -> Self {
        Self { schedules }
    }
}

impl Scheduled for Schedules {
    fn next_after<Z: TimeZone>(&self, after: &DateTime<Z>) -> Option<DateTime<Z>> {
        self.schedules
            .iter()
            .filter_map(|s| s.next_after(after))
            .min()
    }

    fn prev_from<Z: TimeZone>(&self, from: &DateTime<Z>) -> Option<DateTime<Z>> {
        self.schedules
            .iter()
            .filter_map(|s| s.prev_from(from))
            .max()
    }
}

#[cfg(feature = "serde")]
mod parsing {
    use cron::Schedule;
    use serde::de::{SeqAccess, Visitor};
    use serde::ser::SerializeSeq;
    use serde::{Deserializer, Serializer};
    use std::str::FromStr;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Schedule>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SchedulesVisitor;

        impl<'de> Visitor<'de> for SchedulesVisitor {
            type Value = Vec<Schedule>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a list of schedules")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut schedules = Vec::new();
                while let Some(schedule) = seq.next_element::<String>()? {
                    schedules.push(
                        Schedule::from_str(&schedule)
                            .map_err(|e| serde::de::Error::custom(e.to_string()))?,
                    );
                }
                Ok(schedules)
            }
        }

        deserializer.deserialize_seq(SchedulesVisitor)
    }

    pub fn serialize<S>(schedules: &[Schedule], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(schedules.len()))?;
        for schedule in schedules {
            seq.serialize_element(&schedule.to_string())?;
        }
        seq.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_schedules() {
        let schedules = Schedules {
            schedules: vec![
                Schedule::from_str("0 2,4,8,16 16 * * * *").unwrap(),
                Schedule::from_str("0 1 7-18 * * * *").unwrap(),
            ],
        };

        let time = chrono::DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z").unwrap();
        let mut iterator = schedules.after(&time);
        assert_eq!(
            iterator.next().unwrap(),
            chrono::DateTime::parse_from_rfc3339("2020-01-01T07:01:00Z").unwrap()
        );
    }
}
