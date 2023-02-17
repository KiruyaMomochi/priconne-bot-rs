mod scheduled;
mod schedules;

pub use scheduled::Scheduled;
pub use schedules::Schedules;

use chrono::{DateTime, Local, TimeZone, Utc};

/// A function that runs at time specified by schedules
pub struct Action<'a, S: Scheduled, Z: TimeZone> {
    /// Schedule of the action
    scheduled: S,
    /// Last time the action was run
    last_tick: Option<DateTime<Z>>,
    /// Time zone of the action
    time_zone: Z,
    /// Action to run
    run: Box<dyn (FnMut()) + Send + Sync + 'a>,
}

impl<'a, S, Z> Action<'a, S, Z>
where
    S: Scheduled,
    Z: TimeZone,
{
    /// Create a new action with the given schedules and run function
    /// in specified time zone
    pub fn new<T>(scheduled: S, run: T, time_zone: Z) -> Self
    where
        T: (FnMut()) + Send + Sync + 'a,
    {
        Self {
            scheduled,
            last_tick: None,
            run: Box::new(run),
            time_zone,
        }
    }

    /// Check if the action should run now, and run it if needed
    pub fn tick(&mut self) {
        let now = Utc::now().with_timezone(&self.time_zone);
        if self.last_tick.is_none() {
            self.last_tick = Some(now);
            return;
        }
        let last_tick = self.last_tick.as_ref().unwrap();
        let event = self.scheduled.after(last_tick).next().unwrap();
        if event <= now {
            (self.run)();
        }
        self.last_tick = Some(now);
    }
}

impl<'a, S> Action<'a, S, Utc>
where
    S: Scheduled,
{
    /// Create a new action with the given schedules and run function
    /// in UTC time zone
    pub fn new_utc<T>(scheduled: S, run: T) -> Self
    where
        T: (FnMut()) + Send + Sync + 'a,
    {
        Self::new(scheduled, run, Utc)
    }
}

impl<'a, S> Action<'a, S, Local>
where
    S: Scheduled,
{
    /// Create a new action with the given schedules and run function
    /// in local time zone
    pub fn new_local<T>(scheduled: S, run: T) -> Self
    where
        T: (FnMut()) + Send + Sync + 'a,
    {
        Self::new(scheduled, run, Local)
    }
}
