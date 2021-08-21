use chrono::{DateTime, TimeZone, Utc};
use uuid::Uuid;

pub type JobToRun = dyn FnMut(Uuid) + Send + Sync;

struct Action {
    schedules: Schedules,
    last_tick: Option<DateTime<Utc>>,
    job_id: Uuid,
    run: Box<JobToRun>,
}

impl Action {}

struct Schedules {
    schedules: Vec<cron::Schedule>,
}

impl Schedules {
    pub fn new(schedules: Vec<cron::Schedule>) -> Self {
        Self { schedules }
    }

    pub fn upcoming<Z>(&self, timezone: Z) -> SchedulesIterator<Z>
    where
        Z: TimeZone,
    {
        self.after(&chrono::Utc::now().with_timezone(&timezone))
    }

    pub fn after<Z>(&self, after: &DateTime<Z>) -> SchedulesIterator<Z>
    where
        Z: TimeZone,
    {
        SchedulesIterator::<Z>::new(&self.schedules, after)
    }
}

pub struct SchedulesIterator<'a, Z>
where
    Z: TimeZone,
{
    is_done: bool,
    schedule: &'a Vec<cron::Schedule>,
    previous_datetime: DateTime<Z>,
}

impl<'a, Z> SchedulesIterator<'a, Z>
where
    Z: TimeZone,
{
    fn new(schedule: &'a Vec<cron::Schedule>, starting_datetime: &DateTime<Z>) -> Self {
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
    use std::str::FromStr;

    use cron::Schedule;

    use crate::{schedule::Schedules, utils::HOUR};

    #[test]
    fn cron_string() {
        let schedules = Schedules {
            schedules: vec![
                Schedule::from_str("0 2,4,8,16 16 * * * *").unwrap(),
                Schedule::from_str("0 1 7-18 * * * *").unwrap(),
            ],
        };

        let _fuck = |x: i32| async move { x + 1 };

        println!(
            "{:#?}",
            schedules
                .upcoming(chrono::FixedOffset::east(8 * HOUR))
                .take(50)
                .collect::<Vec<_>>()
        );
    }
}
