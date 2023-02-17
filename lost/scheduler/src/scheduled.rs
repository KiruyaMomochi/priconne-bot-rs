use chrono::{DateTime, TimeZone};
use cron::Schedule;

pub trait Scheduled {
    fn next_after<Z: TimeZone>(&self, after: &DateTime<Z>) -> Option<DateTime<Z>>;
    fn prev_from<Z: TimeZone>(&self, from: &DateTime<Z>) -> Option<DateTime<Z>>;

    /// Get the next event after now
    fn upcoming<Z>(&self, timezone: Z) -> ScheduledIterator<'_, Self, Z>
    where
        Z: TimeZone,
        Self: Sized,
    {
        self.after(&chrono::Utc::now().with_timezone(&timezone))
    }

    /// Get the next event after the given time
    fn after<Z>(&self, after: &DateTime<Z>) -> ScheduledIterator<'_, Self, Z>
    where
        Z: TimeZone,
        Self: Sized,
    {
        ScheduledIterator::new(self, after)
    }

    fn upcoming_owned<Z>(&self, timezone: Z) -> OwnedScheduledIterator<Self, Z>
    where
        Z: TimeZone,
        Self: Sized + Clone,
    {
        self.after_owned(&chrono::Utc::now().with_timezone(&timezone))
    }

    fn after_owned<Z>(&self, after: &DateTime<Z>) -> OwnedScheduledIterator<Self, Z>
    where
        Z: TimeZone,
        Self: Sized + Clone,
    {
        OwnedScheduledIterator::new(self.clone(), after)
    }
}

pub struct ScheduledIterator<'a, S, Z>
where
    S: Scheduled + Sized,
    Z: TimeZone,
{
    scheduled: &'a S,
    previous_datetime: Option<DateTime<Z>>,
}

impl<'a, S, Z> ScheduledIterator<'a, S, Z>
where
    S: Scheduled,
    Z: TimeZone,
{
    pub fn new(scheduled: &'a S, starting_datetime: &DateTime<Z>) -> Self {
        Self {
            scheduled,
            previous_datetime: Some(starting_datetime.clone()),
        }
    }
}

impl<'a, S, Z> Iterator for ScheduledIterator<'a, S, Z>
where
    S: Scheduled,
    Z: TimeZone,
{
    type Item = DateTime<Z>;

    fn next(&mut self) -> Option<Self::Item> {
        // By using `take`, the previous datetime is taken out.
        // Avoiding to take twice the same datetime.
        let previous = self.previous_datetime.take()?;
        let next = self.scheduled.next_after(&previous)?;
        self.previous_datetime = Some(next.clone());
        Some(next)
    }
}

impl<'a, S, Z> DoubleEndedIterator for ScheduledIterator<'a, S, Z>
where
    S: Scheduled,
    Z: TimeZone,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        // By using `take`, the previous datetime is taken out.
        // Avoiding to take twice the same datetime.
        let previous = self.previous_datetime.take()?;
        let next = self.scheduled.prev_from(&previous)?;
        self.previous_datetime = Some(next.clone());
        Some(next)
    }
}

pub struct OwnedScheduledIterator<S, Z>
where
    S: Scheduled + Sized,
    Z: TimeZone,
{
    scheduled: S,
    previous_datetime: Option<DateTime<Z>>,
}

impl<S, Z> OwnedScheduledIterator<S, Z>
where
    S: Scheduled,
    Z: TimeZone,
{
    pub fn new(scheduled: S, starting_datetime: &DateTime<Z>) -> Self {
        Self {
            scheduled,
            previous_datetime: Some(starting_datetime.clone()),
        }
    }
}

impl<S, Z> Iterator for OwnedScheduledIterator<S, Z>
where
    S: Scheduled,
    Z: TimeZone,
{
    type Item = DateTime<Z>;

    fn next(&mut self) -> Option<Self::Item> {
        // By using `take`, the previous datetime is taken out.
        // Avoiding to take twice the same datetime.
        let previous = self.previous_datetime.take()?;
        let next = self.scheduled.next_after(&previous)?;
        self.previous_datetime = Some(next.clone());
        Some(next)
    }
}

impl<S, Z> DoubleEndedIterator for OwnedScheduledIterator<S, Z>
where
    S: Scheduled,
    Z: TimeZone,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        // By using `take`, the previous datetime is taken out.
        // Avoiding to take twice the same datetime.
        let previous = self.previous_datetime.take()?;
        let next = self.scheduled.prev_from(&previous)?;
        self.previous_datetime = Some(next.clone());
        Some(next)
    }
}

impl Scheduled for Schedule {
    fn next_after<Z: TimeZone>(&self, after: &DateTime<Z>) -> Option<DateTime<Z>> {
        self.after(after).next()
    }

    fn prev_from<Z: TimeZone>(&self, from: &DateTime<Z>) -> Option<DateTime<Z>> {
        self.after(from).next_back()
    }
}
