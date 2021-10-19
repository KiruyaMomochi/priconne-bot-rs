use chrono::{DateTime, TimeZone, Utc};
use cron::Schedule;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// A function that runs at time specified by schedules
pub struct Action<'a> {
    /// Schedule of the action
    schedules: Schedules,
    /// Last time the action was run
    last_tick: Option<DateTime<Utc>>,
    /// Action to run
    run: Box<dyn (FnMut() -> ()) + Send + Sync + 'a>,
}

impl<'a> Action<'a> {
    /// Create a new action with the given schedules and run function
    pub fn new<T>(schedules: Schedules, run: T) -> Self
    where
        T: (FnMut() -> ()) + Send + Sync + 'a,
    {
        Self {
            schedules,
            last_tick: None,
            run: Box::new(run),
        }
    }

    /// Check if the action should run now, and run it if needed
    pub fn tick(&mut self) {
        let now = Utc::now();
        if self.last_tick.is_none() {
            self.last_tick = Some(now);
            return;
        }
        let last_tick = self.last_tick.unwrap();
        let event = self.schedules.after(&last_tick).next().unwrap();
        if event <= now {
            (self.run)();
        }
        self.last_tick = Some(now);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(transparent)] 
pub struct Schedules {
    /// List of [`Schedule`]
    #[serde(with = "schedules")]
    schedules: Vec<Schedule>,
}

mod schedules {
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
                    schedules.push(Schedule::from_str(&schedule).unwrap());
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

impl TryFrom<Vec<String>> for Schedules {
    type Error = cron::error::Error;

    fn try_from(vec: Vec<String>) -> Result<Self, Self::Error> {
        let mut schedules = Vec::new();
        for s in vec {
            schedules.push(s.parse()?);
        }
        Ok(Self { schedules })
    }
}

impl ToString for Schedules {
    fn to_string(&self) -> String {
        let mut s = String::new();
        for schedule in &self.schedules {
            s.push_str(&schedule.to_string());
            s.push_str("\n");
        }
        s
    }
}

impl Schedules {
    /// Create a new schedules with the given list of schedules
    pub fn new(schedules: Vec<Schedule>) -> Self {
        Self { schedules }
    }

    /// Get the next event after now
    pub fn upcoming<Z>(&self, timezone: Z) -> SchedulesIterator<'_, Z>
    where
        Z: TimeZone,
    {
        self.after(&chrono::Utc::now().with_timezone(&timezone))
    }

    /// Get the next event after the given time
    pub fn after<Z>(&self, after: &DateTime<Z>) -> SchedulesIterator<'_, Z>
    where
        Z: TimeZone,
    {
        SchedulesIterator::<Z>::new(&self.schedules, after)
    }
}

/// Iterator over the schedules
pub struct SchedulesIterator<'a, Z>
where
    Z: TimeZone,
{
    is_done: bool,
    schedule: &'a Vec<Schedule>,
    previous_datetime: DateTime<Z>,
}

impl<'a, Z> SchedulesIterator<'a, Z>
where
    Z: TimeZone,
{
    fn new(schedule: &'a Vec<Schedule>, starting_datetime: &DateTime<Z>) -> Self {
        Self {
            is_done: false,
            previous_datetime: starting_datetime.clone(),
            schedule,
        }
    }
}

impl<'a, Z> Iterator for SchedulesIterator<'a, Z>
where
    Z: TimeZone,
{
    type Item = DateTime<Z>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done {
            return None;
        }
        let mut result = None;
        for schedule in self.schedule {
            if let Some(next_datetime) = schedule.after::<Z>(&self.previous_datetime).next() {
                if let Some(datetime) = &result {
                    if &next_datetime < datetime {
                        result = Some(next_datetime);
                    }
                } else {
                    result = Some(next_datetime);
                }
            }
        }

        if result.is_some() {
            self.previous_datetime = result.as_ref().unwrap().clone();
        } else {
            self.is_done = true;
        }

        result
    }
}

impl<'a, Z> DoubleEndedIterator for SchedulesIterator<'a, Z>
where
    Z: TimeZone,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.is_done {
            return None;
        }
        let mut result = None;
        for schedule in self.schedule {
            if let Some(next_datetime) = schedule.after::<Z>(&self.previous_datetime).next_back() {
                if let Some(datetime) = &result {
                    if &next_datetime < datetime {
                        result = Some(next_datetime);
                    }
                }
            }
        }

        if result.is_some() {
            self.previous_datetime = result.as_ref().unwrap().clone();
        } else {
            self.is_done = true;
        }

        result
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
