// A runtime-adjustable "how fast does the simulation tick" dial, shared
// between the worker thread (which sleeps between ticks) and the Bevy
// render thread (which reads/cycles it in response to key presses).
//
// Slowing this down necessarily slows down training itself, not just the
// visualization -- the worker thread's tick loop *is* what's computing
// fitness, so there's no way to decouple "how fast it looks" from "how
// fast it runs" without simulating twice.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

pub const PRESETS: &[(&str, u32)] = &[
    ("Max", 0),
    ("4x", 4_166),
    ("2x", 8_333),
    ("1x", 16_666),
    ("0.5x", 33_333),
    ("0.25x", 66_666),
    ("0.1x", 166_666),
];

const DEFAULT_PRESET: usize = 3; // "1x" -- realtime, watchable out of the box

#[derive(Clone)]
pub struct SimSpeed(Arc<AtomicU32>);

impl SimSpeed {
    pub fn new() -> Self {
        SimSpeed(Arc::new(AtomicU32::new(DEFAULT_PRESET as u32)))
    }

    pub fn delay(&self) -> Duration {
        let idx = self.0.load(Ordering::Relaxed) as usize;
        Duration::from_micros(PRESETS[idx].1 as u64)
    }

    pub fn label(&self) -> &'static str {
        let idx = self.0.load(Ordering::Relaxed) as usize;
        PRESETS[idx].0
    }

    pub fn faster(&self) {
        let idx = self.0.load(Ordering::Relaxed) as usize;
        if idx > 0 {
            self.0.store(idx as u32 - 1, Ordering::Relaxed);
        }
    }

    pub fn slower(&self) {
        let idx = self.0.load(Ordering::Relaxed) as usize;
        if idx + 1 < PRESETS.len() {
            self.0.store(idx as u32 + 1, Ordering::Relaxed);
        }
    }
}
