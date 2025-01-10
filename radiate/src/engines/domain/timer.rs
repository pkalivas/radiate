use std::time::Instant;

/// A simple timer that can be used to measure the time it takes to perform
/// an operation. The timer can be stopped and started, and the duration
/// can be retrieved at any time.
///
/// This is here just to make it easier to measure time without having to
/// deal with the `Instant` struct directly.
pub struct Timer {
    start: Instant,
    end: Instant,
    stopped: bool,
}

impl Timer {
    /// Create a new instance of the `Timer`.
    pub fn new() -> Timer {
        Timer {
            start: Instant::now(),
            end: Instant::now(),
            stopped: false,
        }
    }

    pub fn stop(&mut self) {
        self.end = Instant::now();
        self.stopped = true;
    }

    pub fn duration(&self) -> std::time::Duration {
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
