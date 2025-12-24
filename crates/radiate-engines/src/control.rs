use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicBool, Ordering},
};

#[derive(Debug)]
struct State {
    paused: bool,
    permits: usize,
}

#[derive(Clone)]
pub struct EngineControl {
    stop_flag: Arc<AtomicBool>,
    inner: Arc<(Mutex<State>, Condvar)>,
}

impl EngineControl {
    pub fn new() -> Self {
        Self {
            stop_flag: Arc::new(AtomicBool::new(false)),
            inner: Arc::new((
                Mutex::new(State {
                    paused: false,
                    permits: 0,
                }),
                Condvar::new(),
            )),
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
        self.stop_flag.store(true, Ordering::SeqCst);
        // wake anything blocked
        self.set_paused(true);
    }

    #[inline]
    pub fn is_stopped(&self) -> bool {
        self.stop_flag.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn stop_flag(&self) -> Arc<AtomicBool> {
        self.stop_flag.clone()
    }

    // ---- pause/step ----
    #[inline]
    pub fn set_paused(&self, paused: bool) {
        let (lock, cv) = &*self.inner;
        let mut st = lock.lock().unwrap();
        st.paused = paused;
        if !paused {
            st.permits = 0; // permits irrelevant when running
        }
        cv.notify_all();
    }

    /// Toggle pause. Returns new paused state.
    #[inline]
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

    #[inline]
    pub fn step_once(&self) {
        let (lock, cv) = &*self.inner;
        let mut st = lock.lock().unwrap();
        st.paused = true;
        st.permits += 1;
        cv.notify_all();
    }

    /// Called by engine thread before computing next epoch.
    #[inline]
    pub fn wait_before_step(&self) {
        let (lock, cv) = &*self.inner;
        let mut st = lock.lock().unwrap();

        while !self.stop_flag.load(Ordering::Relaxed) {
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

    /// Optional: expose for UI display
    #[inline]
    pub fn is_paused(&self) -> bool {
        let (lock, _) = &*self.inner;
        lock.lock().unwrap().paused
    }
}
