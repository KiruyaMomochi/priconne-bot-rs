use chrono::{DateTime, TimeZone, Utc};
use cron::Schedule;

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

pub struct Schedules {
    /// List of [`Schedule`]
    schedules: Vec<Schedule>,
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
