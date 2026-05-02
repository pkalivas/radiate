use std::time::{Duration, Instant};

pub struct RunState {
    pub engine: bool,
    pub ui: bool,
    pub paused: bool,
    pub last_render: Option<Instant>,
    pub render_interval: Duration,
}

impl Default for RunState {
    fn default() -> Self {
        Self {
            engine: false,
            ui: true,
            paused: false,
            last_render: None,
            render_interval: Duration::from_millis(500),
        }
    }
}
