use std::time::{Duration, SystemTime, SystemTimeError};

pub struct AppTime {
    start_time: SystemTime,
    pub elapsed: Duration,
}

impl AppTime {
    pub fn new() -> Self {
        Self {
            start_time: SystemTime::now(),
            elapsed: Duration::default(),
        }
    }

    pub fn update(&mut self) -> Result<(), SystemTimeError> {
        let now = SystemTime::now();
        self.elapsed = now.duration_since(self.start_time)?;
        Ok(())
    }
}