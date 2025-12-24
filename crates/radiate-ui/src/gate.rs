use std::sync::{Arc, Condvar, Mutex, atomic::AtomicBool, atomic::Ordering};

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

    pub fn pair(paused: bool) -> (Self, Self) {
        let gate = Self::new();
        gate.set_paused(paused);
        (gate.clone(), gate)
    }

    pub fn set_paused(&self, paused: bool) {
        let (lock, cv) = &*self.inner;
        let mut st = lock.lock().unwrap();
        st.paused = paused;
        if !paused {
            st.permits = 0; // permits irrelevant when running
        }
        cv.notify_all();
    }

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

    pub fn step_once(&self) {
        let (lock, cv) = &*self.inner;
        let mut st = lock.lock().unwrap();
        st.paused = true; // stepping implies paused mode
        st.permits += 1; // allow exactly one epoch
        cv.notify_all();
    }

    pub fn wait_for_permit(&self, stop_flag: &AtomicBool) {
        let (lock, cv) = &*self.inner;
        let mut st = lock.lock().unwrap();

        while !stop_flag.load(Ordering::Relaxed) {
            if !st.paused {
                return; // running freely
            }
            if st.permits > 0 {
                st.permits -= 1; // consume one step
                return;
            }
            st = cv.wait(st).unwrap();
        }
    }
}

// #[derive(Clone)]
// pub struct PauseGate {
//     paused: Arc<(Mutex<bool>, Condvar)>,
// }

// impl PauseGate {
//     fn new(paused: bool) -> Self {
//         Self {
//             paused: Arc::new((Mutex::new(paused), Condvar::new())),
//         }
//     }

//     pub fn pair(paused: bool) -> (Self, Self) {
//         let gate = Self::new(paused);
//         (gate.clone(), gate)
//     }

//     pub fn set(&self, paused: bool) {
//         let (lock, cv) = &*self.paused;
//         let mut p = lock.lock().unwrap();
//         *p = paused;
//         cv.notify_all();
//     }

//     pub fn toggle(&self) -> bool {
//         let (lock, cv) = &*self.paused;
//         let mut p = lock.lock().unwrap();
//         *p = !*p;
//         let now = *p;
//         cv.notify_all();
//         now
//     }

//     pub fn wait_if_paused(&self, stop_flag: &AtomicBool) {
//         use std::sync::atomic::Ordering;
//         let (lock, cv) = &*self.paused;
//         let mut p = lock.lock().unwrap();

//         while *p && !stop_flag.load(Ordering::Relaxed) {
//             p = cv.wait(p).unwrap();
//         }
//     }

//     // pub fn is_paused(&self) -> bool {
//     //     let (lock, _) = &*self.paused;
//     //     *lock.lock().unwrap()
//     // }
// }
