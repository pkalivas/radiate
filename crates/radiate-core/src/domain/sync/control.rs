use super::{Flag, FlagReader};
use std::{
    ops::ControlFlow,
    sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    Proceed,
    Stopped,
}

#[derive(Debug, Default)]
struct State {
    permits: usize,
}

#[derive(Debug, Default)]
struct Shared {
    stopped: Flag,
    paused: Flag,
    state: Mutex<State>,
    cv: Condvar,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn set_paused(&self, st: &mut State, paused: bool) {
        if !paused {
            st.permits = 0;
        }

        self.paused.store(paused);
    }
}

#[derive(Clone, Debug, Default)]
pub struct ThreadSync {
    inner: Arc<Shared>,
}

impl ThreadSync {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pair() -> (Self, Self) {
        let ctl = Self::new();
        (ctl.clone(), ctl)
    }

    pub fn stop_flag(&self) -> FlagReader {
        self.inner.stopped.reader()
    }

    pub fn pause_flag(&self) -> FlagReader {
        self.inner.paused.reader()
    }

    #[inline]
    pub fn is_stopped(&self) -> bool {
        self.inner.stopped.is_raised()
    }

    #[inline]
    pub fn is_paused(&self) -> bool {
        self.inner.paused.is_raised()
    }

    pub fn stop(&self) {
        if self.inner.stopped.raise() {
            let _guard = self.inner.lock();
            self.inner.cv.notify_all();
        }
    }

    pub fn set_paused(&self, paused: bool) {
        let mut st = self.inner.lock();
        self.inner.set_paused(&mut st, paused);
        drop(st);
        self.inner.cv.notify_all();
    }

    pub fn toggle_pause(&self) -> bool {
        let mut st = self.inner.lock();
        let now = !self.inner.paused.is_raised();
        self.inner.set_paused(&mut st, now);
        drop(st);
        self.inner.cv.notify_all();
        now
    }

    pub fn step_n(&self, n: usize) {
        let mut st = self.inner.lock();
        self.inner.set_paused(&mut st, true);
        st.permits += n;
        drop(st);
        self.inner.cv.notify_all();
    }

    pub fn step_once(&self) {
        self.step_n(1);
    }

    pub fn poll(&self) -> Option<ControlFlow<()>> {
        self.poll_internal(&mut self.inner.lock())
    }

    pub fn wait(&self) -> ControlFlow<()> {
        let mut st = self.inner.lock();
        loop {
            if let Some(gate) = self.poll_internal(&mut st) {
                return gate;
            }

            st = self
                .inner
                .cv
                .wait(st)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }

    #[inline]
    fn poll_internal(&self, state: &mut State) -> Option<ControlFlow<()>> {
        if self.inner.stopped.is_raised() {
            return Some(ControlFlow::Break(()));
        }

        if !self.inner.paused.is_raised() {
            return Some(ControlFlow::Continue(()));
        }

        if state.permits > 0 {
            state.permits -= 1;
            return Some(ControlFlow::Continue(()));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::{Duration, Instant};

    fn wait_until(timeout: Duration, mut f: impl FnMut() -> bool) -> bool {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if f() {
                return true;
            }
            thread::sleep(Duration::from_millis(1));
        }
        f()
    }

    fn spawn_worker(ctl: &ThreadSync, max: usize) -> (Arc<AtomicUsize>, thread::JoinHandle<usize>) {
        let count = Arc::new(AtomicUsize::new(0));
        let (c, ctl) = (Arc::clone(&count), ctl.clone());
        let h = thread::spawn(move || {
            for _ in 0..max {
                match ctl.wait() {
                    ControlFlow::Break(()) => break,
                    ControlFlow::Continue(()) => {
                        c.fetch_add(1, Ordering::SeqCst);
                    }
                }
            }
            c.load(Ordering::SeqCst)
        });
        (count, h)
    }

    #[test]
    fn step_n_runs_exactly_n_then_stop_exits() {
        let ctl = ThreadSync::new();
        ctl.step_n(10);
        let (count, h) = spawn_worker(&ctl, 15);

        assert!(wait_until(Duration::from_secs(5), || count
            .load(Ordering::SeqCst)
            == 10));
        thread::sleep(Duration::from_millis(50));
        assert_eq!(count.load(Ordering::SeqCst), 10, "ran past permits");

        ctl.stop();
        assert_eq!(h.join().unwrap(), 10, "proceeded after stop");
    }

    #[test]
    fn stop_does_not_change_pause_state() {
        let ctl = ThreadSync::new();
        ctl.stop();
        assert!(!ctl.is_paused());
        assert_eq!(ctl.wait(), ControlFlow::Break(()));
    }

    #[test]
    fn try_wait_consumes_permits_then_would_block() {
        let ctl = ThreadSync::new();
        assert_eq!(ctl.poll(), Some(ControlFlow::Continue(())));
        ctl.step_n(2);
        assert_eq!(ctl.poll(), Some(ControlFlow::Continue(())));
        assert_eq!(ctl.poll(), Some(ControlFlow::Continue(())));
        assert_eq!(ctl.poll(), None);
        ctl.stop();
        assert_eq!(ctl.poll(), Some(ControlFlow::Break(())));
    }

    #[test]
    fn parked_waiter_released_by_resume_and_by_stop() {
        let ctl = ThreadSync::new();
        ctl.set_paused(true);
        let c2 = ctl.clone();
        let h = thread::spawn(move || c2.wait());
        thread::sleep(Duration::from_millis(50));
        ctl.set_paused(false);
        assert_eq!(h.join().unwrap(), ControlFlow::Continue(()));

        ctl.set_paused(true);
        let c3 = ctl.clone();
        let h = thread::spawn(move || c3.wait());
        thread::sleep(Duration::from_millis(50));
        ctl.stop();
        assert_eq!(h.join().unwrap(), ControlFlow::Break(()));
    }

    #[test]
    fn resume_clears_permits() {
        let ctl = ThreadSync::new();
        ctl.step_n(5);
        ctl.set_paused(false);
        ctl.set_paused(true);
        assert_eq!(ctl.inner.lock().permits, 0);
    }

    #[test]
    fn lent_signals_track_state() {
        let ctl = ThreadSync::new();
        let (stop, pause) = (ctl.stop_flag(), ctl.pause_flag());
        ctl.step_once();
        assert!(pause.is_raised());
        ctl.set_paused(false);
        assert!(!pause.is_raised());
        ctl.stop();
        assert!(stop.is_raised());
    }

    #[test]
    fn a_worker_can_loop_on_a_lent_signal() {
        let ctl = ThreadSync::new();
        let stop = ctl.stop_flag();
        let h = thread::spawn(move || {
            let mut n = 0u64;
            while !stop.is_raised() {
                n += 1;
                thread::yield_now();
            }
            n
        });
        thread::sleep(Duration::from_millis(20));
        ctl.stop();
        assert!(h.join().unwrap() > 0);
    }
}

// use std::sync::{
//     Arc, Condvar, Mutex, MutexGuard, PoisonError,
//     atomic::{AtomicBool, Ordering},
// };

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum Gate {
//     Proceed,
//     Stopped,
// }

// #[derive(Debug, Default)]
// struct State {
//     paused: bool,
//     permits: usize,
// }

// impl State {
//     fn set_paused(&mut self, paused: bool) {
//         self.paused = paused;
//         if !paused {
//             self.permits = 0;
//         }
//     }
// }

// #[derive(Debug, Default)]
// struct Shared {
//     stopped: AtomicBool,
//     state: Mutex<State>,
//     cv: Condvar,
// }

// impl Shared {
//     fn lock(&self) -> MutexGuard<'_, State> {
//         self.state.lock().unwrap_or_else(PoisonError::into_inner)
//     }
// }

// #[derive(Clone, Debug)]
// pub struct StopToken(Arc<Shared>);

// impl StopToken {
//     #[inline]
//     pub fn is_stopped(&self) -> bool {
//         self.0.stopped.load(Ordering::Relaxed)
//     }
// }

// #[derive(Clone, Debug, Default)]
// pub struct ThreadSync {
//     shared: Arc<Shared>,
// }

// impl ThreadSync {
//     pub fn new() -> Self {
//         Self::default()
//     }

//     pub fn pair() -> (Self, Self) {
//         let ctl = Self::new();
//         (ctl.clone(), ctl)
//     }

//     #[inline]
//     pub fn is_stopped(&self) -> bool {
//         self.shared.stopped.load(Ordering::Relaxed)
//     }

//     pub fn is_paused(&self) -> bool {
//         self.shared.lock().paused
//     }

//     pub fn stop_token(&self) -> StopToken {
//         StopToken(Arc::clone(&self.shared))
//     }

//     pub fn stop(&self) {
//         self.shared.stopped.store(true, Ordering::Release);
//         let _guard = self.shared.lock();
//         self.shared.cv.notify_all();
//     }

//     pub fn set_paused(&self, paused: bool) {
//         self.shared.lock().set_paused(paused);
//         self.shared.cv.notify_all();
//     }

//     pub fn toggle_pause(&self) -> bool {
//         let mut st = self.shared.lock();
//         let now = !st.paused;
//         st.set_paused(now);
//         drop(st);
//         self.shared.cv.notify_all();
//         now
//     }

//     pub fn step_n(&self, n: usize) {
//         let mut st = self.shared.lock();
//         st.paused = true;
//         st.permits += n;
//         drop(st);
//         self.shared.cv.notify_all();
//     }

//     pub fn step_once(&self) {
//         self.step_n(1);
//     }

//     pub fn try_wait(&self) -> Option<Gate> {
//         self.poll(&mut self.shared.lock())
//     }

//     pub fn wait(&self) -> Gate {
//         let mut st = self.shared.lock();
//         loop {
//             if let Some(gate) = self.poll(&mut st) {
//                 return gate;
//             }

//             st = self
//                 .shared
//                 .cv
//                 .wait(st)
//                 .unwrap_or_else(PoisonError::into_inner);
//         }
//     }

//     #[inline]
//     fn poll(&self, st: &mut State) -> Option<Gate> {
//         if self.shared.stopped.load(Ordering::Acquire) {
//             return Some(Gate::Stopped);
//         }

//         if !st.paused {
//             return Some(Gate::Proceed);
//         }

//         if st.permits > 0 {
//             st.permits -= 1;
//             return Some(Gate::Proceed);
//         }

//         None
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::sync::atomic::AtomicUsize;
//     use std::thread;
//     use std::time::{Duration, Instant};

//     fn wait_until(timeout: Duration, mut f: impl FnMut() -> bool) -> bool {
//         let deadline = Instant::now() + timeout;
//         while Instant::now() < deadline {
//             if f() {
//                 return true;
//             }
//             thread::sleep(Duration::from_millis(1));
//         }
//         f()
//     }

//     /// Spawns a worker loop; returns (count, handle -> number of gens run).
//     fn spawn_worker(ctl: &ThreadSync, max: usize) -> (Arc<AtomicUsize>, thread::JoinHandle<usize>) {
//         let count = Arc::new(AtomicUsize::new(0));
//         let (c, ctl) = (Arc::clone(&count), ctl.clone());
//         let h = thread::spawn(move || {
//             for _ in 0..max {
//                 match ctl.wait() {
//                     Gate::Stopped => break,
//                     Gate::Proceed => {
//                         c.fetch_add(1, Ordering::SeqCst);
//                     }
//                 }
//             }
//             c.load(Ordering::SeqCst)
//         });
//         (count, h)
//     }

//     #[test]
//     fn step_n_runs_exactly_n_then_stop_exits() {
//         let ctl = ThreadSync::new();
//         ctl.step_n(10);
//         let (count, h) = spawn_worker(&ctl, 15);

//         assert!(wait_until(Duration::from_secs(5), || count
//             .load(Ordering::SeqCst)
//             == 10));
//         thread::sleep(Duration::from_millis(50));
//         assert_eq!(count.load(Ordering::SeqCst), 10, "ran past permits");

//         ctl.stop();
//         assert_eq!(h.join().unwrap(), 10, "proceeded after stop");
//     }

//     #[test]
//     fn stop_does_not_change_pause_state() {
//         let ctl = ThreadSync::new();
//         ctl.stop();
//         assert!(!ctl.is_paused());
//         assert_eq!(ctl.wait(), Gate::Stopped);
//     }

//     #[test]
//     fn try_wait_consumes_permits_then_would_block() {
//         let ctl = ThreadSync::new();
//         assert_eq!(ctl.try_wait(), Some(Gate::Proceed)); // running
//         ctl.step_n(2);
//         assert_eq!(ctl.try_wait(), Some(Gate::Proceed));
//         assert_eq!(ctl.try_wait(), Some(Gate::Proceed));
//         assert_eq!(ctl.try_wait(), None);
//         ctl.stop();
//         assert_eq!(ctl.try_wait(), Some(Gate::Stopped));
//     }

//     #[test]
//     fn parked_waiter_released_by_resume_and_by_stop() {
//         let ctl = ThreadSync::new();
//         ctl.set_paused(true);
//         let c2 = ctl.clone();
//         let h = thread::spawn(move || c2.wait());
//         thread::sleep(Duration::from_millis(50));
//         ctl.set_paused(false);
//         assert_eq!(h.join().unwrap(), Gate::Proceed);

//         ctl.set_paused(true);
//         let c3 = ctl.clone();
//         let h = thread::spawn(move || c3.wait());
//         thread::sleep(Duration::from_millis(50));
//         ctl.stop();
//         assert_eq!(h.join().unwrap(), Gate::Stopped);
//     }

//     #[test]
//     fn resume_clears_permits() {
//         let ctl = ThreadSync::new();
//         ctl.step_n(5);
//         ctl.set_paused(false);
//         ctl.set_paused(true);
//         assert_eq!(ctl.shared.lock().permits, 0);
//     }
// }

// // use std::sync::{
// //     Arc, Condvar, Mutex,
// //     atomic::{AtomicBool, Ordering},
// // };

// // #[derive(Debug, Default)]
// // struct State {
// //     paused: bool,
// //     permits: usize,
// // }

// // #[derive(Clone, Default)]
// // pub struct ThreadSync {
// //     stop_flag: Arc<AtomicBool>,
// //     inner: Arc<(Mutex<State>, Condvar)>,
// // }

// // impl ThreadSync {
// //     pub fn new() -> Self {
// //         Self {
// //             stop_flag: Arc::new(AtomicBool::new(false)),
// //             inner: Arc::new((
// //                 Mutex::new(State {
// //                     paused: false,
// //                     permits: 0,
// //                 }),
// //                 Condvar::new(),
// //             )),
// //         }
// //     }

// //     pub fn pair() -> (Self, Self) {
// //         let ctl = Self::new();
// //         (ctl.clone(), ctl)
// //     }

// //     #[inline]
// //     pub fn stop(&self) {
// //         self.stop_flag.store(true, Ordering::SeqCst);
// //         // wake anything blocked
// //         self.set_paused(true);
// //     }

// //     #[inline]
// //     pub fn is_stopped(&self) -> bool {
// //         self.stop_flag.load(Ordering::Relaxed)
// //     }

// //     #[inline]
// //     pub fn stop_flag(&self) -> Arc<AtomicBool> {
// //         self.stop_flag.clone()
// //     }

// //     #[inline]
// //     pub fn set_paused(&self, paused: bool) {
// //         let (lock, cv) = &*self.inner;
// //         let mut st = lock.lock().unwrap();
// //         st.paused = paused;
// //         if !paused {
// //             st.permits = 0; // permits irrelevant when running
// //         }
// //         cv.notify_all();
// //     }

// //     #[inline]
// //     pub fn toggle_pause(&self) -> bool {
// //         let (lock, cv) = &*self.inner;
// //         let mut st = lock.lock().unwrap();
// //         st.paused = !st.paused;
// //         if !st.paused {
// //             st.permits = 0;
// //         }
// //         let now = st.paused;
// //         cv.notify_all();
// //         now
// //     }

// //     #[inline]
// //     pub fn step_once(&self) {
// //         self.step_n(1);
// //     }

// //     #[inline]
// //     pub fn step_n(&self, n: usize) {
// //         let (lock, cv) = &*self.inner;
// //         let mut st = lock.lock().unwrap();
// //         st.paused = true;
// //         st.permits += n;
// //         cv.notify_all();
// //     }

// //     #[inline]
// //     pub fn wait(&self) {
// //         let (lock, cv) = &*self.inner;
// //         let mut st = lock.lock().unwrap();

// //         while !self.stop_flag.load(Ordering::Relaxed) {
// //             if !st.paused {
// //                 return;
// //             }

// //             if st.permits > 0 {
// //                 st.permits -= 1;
// //                 return;
// //             }

// //             st = cv.wait(st).unwrap();
// //         }
// //     }

// //     #[inline]
// //     pub fn is_paused(&self) -> bool {
// //         let (lock, _) = &*self.inner;
// //         lock.lock().unwrap().paused
// //     }
// // }

// // #[cfg(test)]
// // mod diag_tests {
// //     use super::*;
// //     use std::sync::atomic::AtomicUsize;
// //     use std::time::Duration;

// //     #[test]
// //     fn step_n_blocks_after_permits_exhausted() {
// //         let control = ThreadSync::new();
// //         control.step_n(10);

// //         let count = Arc::new(AtomicUsize::new(0));
// //         let count2 = Arc::clone(&count);
// //         let control2 = control.clone();

// //         let handle = std::thread::spawn(move || {
// //             for _ in 0..15 {
// //                 control2.wait();
// //                 count2.fetch_add(1, Ordering::SeqCst);
// //             }
// //         });

// //         std::thread::sleep(Duration::from_millis(300));
// //         let progressed = count.load(Ordering::SeqCst);
// //         println!("progressed before stop: {progressed}");
// //         control.stop();
// //         handle.join().unwrap();
// //         let after_stop = count.load(Ordering::SeqCst);
// //         println!("progressed after stop: {after_stop}");

// //         assert_eq!(
// //             progressed, 10,
// //             "expected exactly 10 waits to return before blocking"
// //         );
// //     }
// // }
