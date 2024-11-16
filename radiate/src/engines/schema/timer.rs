use std::time::Instant;

pub struct Timer {
    start: Instant,
    end: Instant,
    stopped: bool,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            start: Instant::now(),
            end: Instant::now(),
            stopped: false,
        }
    }

    pub fn start(&mut self) {
        self.start = Instant::now();
        self.end = Instant::now();
    }

    pub fn stop(&mut self) {
        self.end = Instant::now();
        self.stopped = true;
    }

    pub fn elapsed(&self) -> std::time::Duration {
        if !self.stopped {
            return self.start.elapsed();
        }

        self.end.duration_since(self.start)
    }
}

impl Clone for Timer {
    fn clone(&self) -> Self {
        Timer {
            start: self.start,
            end: self.end,
            stopped: self.stopped,
        }
    }
}
