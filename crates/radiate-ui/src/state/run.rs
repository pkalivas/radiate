use std::time::{Duration, Instant};

use radiate_engines::Subscription;

pub struct RunState {
    pub engine: bool,
    pub ui: bool,
    pub paused: bool,
    pub started: bool,
    pub last_render: Option<Instant>,
    pub render_interval: Duration,
    pub render_count: usize,
    pub event_count: usize,
    pub subscriptions: Vec<Subscription>,
}

impl Default for RunState {
    fn default() -> Self {
        Self {
            engine: false,
            ui: true,
            paused: false,
            started: false,
            last_render: None,
            render_interval: Duration::from_millis(500),
            render_count: 0,
            event_count: 0,
            subscriptions: Vec::new(),
        }
    }
}
