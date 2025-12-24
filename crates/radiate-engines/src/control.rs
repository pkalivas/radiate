use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone)]
pub struct StepGate {
    inner: Arc<(Mutex<State>, Condvar)>,
}

#[derive(Debug)]
struct State {
    paused: bool,
    permits: usize, // how many epochs are allowed while paused
}

impl StepGate {
    pub fn new() -> Self {
        Self {
            inner: Arc::new((
                Mutex::new(State {
                    paused: false,
                    permits: 0,
                }),
                Condvar::new(),
            )),
        }
    }

    /// Pause/resume explicitly.
    pub fn set_paused(&self, paused: bool) {
        let (lock, cv) = &*self.inner;
        let mut st = lock.lock().unwrap();
        st.paused = paused;
        if !paused {
            st.permits = 0; // permits irrelevant when running
        }
        cv.notify_all();
    }

    /// Toggle paused state. Returns the new paused state.
    pub fn toggle_pause(&self) -> bool {
        let (lock, cv) = &*self.inner;
        let mut st = lock.lock().unwrap();
        st.paused = !st.paused;
        if !st.paused {
            st.permits = 0;
        }
        let now = st.paused;
        cv.notify_all();
        now
    }

    /// Allow exactly one `next()` to proceed, leaving the gate paused afterwards.
    pub fn step_once(&self) {
        let (lock, cv) = &*self.inner;
        let mut st = lock.lock().unwrap();
        st.paused = true;
        st.permits += 1;
        cv.notify_all();
    }

    /// Wait until allowed to proceed:
    /// - If not paused => return immediately.
    /// - If paused with permits => consume 1 permit and return.
    /// - Else => block on the condvar.
    pub fn wait_for_permit(&self, stop_flag: &AtomicBool) {
        let (lock, cv) = &*self.inner;
        let mut st = lock.lock().unwrap();

        while !stop_flag.load(Ordering::Relaxed) {
            if !st.paused {
                return;
            }

            if st.permits > 0 {
                st.permits -= 1;
                return;
            }

            st = cv.wait(st).unwrap();
        }
    }

    pub fn is_paused(&self) -> bool {
        let (lock, _) = &*self.inner;
        lock.lock().unwrap().paused
    }
}

#[derive(Clone)]
pub struct EngineControl {
    stop: Arc<AtomicBool>,
    gate: StepGate,
}

impl EngineControl {
    pub fn new() -> Self {
        Self {
            stop: Arc::new(AtomicBool::new(false)),
            gate: StepGate::new(),
        }
    }

    /// Create two clones for separate threads (convenience).
    pub fn pair() -> (Self, Self) {
        let ctl = Self::new();
        (ctl.clone(), ctl)
    }

    // ---- stop ----
    #[inline]
    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
        // wake anything blocked
        self.gate.set_paused(false);
    }

    #[inline]
    pub fn is_stopped(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn stop_flag(&self) -> Arc<AtomicBool> {
        self.stop.clone()
    }

    // ---- pause/step ----
    #[inline]
    pub fn set_paused(&self, paused: bool) {
        self.gate.set_paused(paused);
    }

    /// Toggle pause. Returns new paused state.
    #[inline]
    pub fn toggle_pause(&self) -> bool {
        self.gate.toggle_pause()
    }

    #[inline]
    pub fn step_once(&self) {
        self.gate.step_once()
    }

    /// Called by engine thread before computing next epoch.
    #[inline]
    pub fn wait_before_step(&self) {
        self.gate.wait_for_permit(&self.stop);
    }

    /// Optional: expose for UI display
    #[inline]
    pub fn is_paused(&self) -> bool {
        self.gate.is_paused()
    }
}
