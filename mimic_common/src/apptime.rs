use std::time::{Duration, SystemTime, SystemTimeError};

pub struct AppTime {
    game_start_time: SystemTime,
    last_frame_start_time: SystemTime,
    pub elapsed_since_game_start: Duration,
    pub delta_time: Duration,
    pub frame: u64,
}

impl AppTime {
    pub fn new() -> Self {
        let now = SystemTime::now();
        Self {
            game_start_time: now,
            last_frame_start_time: now,
            elapsed_since_game_start: Duration::default(),
            delta_time: Duration::default(),
            frame: 0,
        }
    }

    pub fn update(&mut self) -> Result<(), SystemTimeError> {
        let now = SystemTime::now();
        self.elapsed_since_game_start = now.duration_since(self.game_start_time)?;
        self.delta_time = now.duration_since(self.last_frame_start_time)?;
        self.frame += 1;
        self.last_frame_start_time = now;
        Ok(())
    }
}
