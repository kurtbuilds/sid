use std::time::{Duration, SystemTime};
use crate::{Label, NoLabel, Sid};

pub struct MonotonicGenerator<T = NoLabel> {
    previous: Sid<T>,
}

impl<T: Label> MonotonicGenerator<T> {
    pub fn new() -> Self {
        Self {
            previous: Sid::<T>::null(),
        }
    }

    pub fn generate(&mut self) -> Sid<T> {
        let now = SystemTime::now();
        self.generate_from_datetime(now)
    }

    pub fn generate_from_datetime(&mut self, datetime: SystemTime) -> Sid<T> {
        self.generate_from_datetime_with_source(datetime, &mut rand::thread_rng())
    }

    pub fn generate_from_datetime_with_source<R>(
        &mut self,
        datetime: SystemTime,
        source: &mut R,
    ) -> Sid<T>
        where
            R: rand::Rng,
    {
        let last_ms = self.previous.timestamp();
        // maybe time went backward, or it is the same ms.
        // increment instead of generating a new random so that it is monotonic
        let ts = datetime
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis() as u64;
        let next = if ts <= last_ms {
            self.previous.increment()
        } else {
            Sid::from_timestamp_with_rng(ts, source)
        };
        self.previous = next.clone();
        next
    }
}