use super::{EventStream, stream::Shared};
use radiate_core::{Executor, error::RadiateResult};
use std::{
    cmp::Ordering as Cmp,
    collections::BinaryHeap,
    sync::{
        Arc, Condvar, Mutex, Weak,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

/// Cancels a scheduled event. Clones share the same timer.
#[derive(Clone)]
pub struct Timer(Arc<AtomicBool>);

impl Timer {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    fn cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

/// Builds and publishes the event when it fires. Takes the stream as an
/// argument rather than capturing it, so a timer never keeps the stream alive.
pub type Job = Box<dyn FnMut(&EventStream) -> RadiateResult<()> + Send>;

struct Entry {
    due: quanta::Instant,
    period: Option<Duration>,
    /// Breaks ties on `due`, so timers due at the same instant fire in the
    /// order they were scheduled.
    seq: u64,
    timer: Timer,
    job: Job,
}

// BinaryHeap is a max-heap. Reverse the order so the earliest deadline is on top.
impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Cmp {
        (other.due, other.seq).cmp(&(self.due, self.seq))
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Cmp> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.due == other.due && self.seq == other.seq
    }
}

impl Eq for Entry {}

#[derive(Default)]
struct State {
    entries: BinaryHeap<Entry>,
    /// Set once by [`Scheduler::close`]. After that nothing fires and nothing
    /// new is accepted.
    closed: bool,
    seq: u64,
    thread: Option<JoinHandle<()>>,
}

/// The scheduler's own state, shared with its thread. Kept apart from the
/// stream's [`Shared`], so the thread can sleep on `wake` without holding the
/// stream alive.
#[derive(Default)]
struct Inner {
    state: Mutex<State>,
    wake: Condvar,
}

/// Timers for one stream. Open from creation until [`close`](Self::close).
///
/// The thread starts with the first timer, so a stream that never schedules
/// anything pays nothing. Dropping the scheduler (with the stream) stops it.
///
/// A timer can fire before the stream is opened, the same as any other
/// publish made before then.
#[derive(Default)]
pub(super) struct Scheduler(Arc<Inner>);

impl Scheduler {
    /// Run `job` after `delay`, then every `period` if one is given. Once the
    /// scheduler is closed, this returns an already cancelled timer and drops
    /// `job` without running it.
    pub(super) fn schedule(
        &self,
        stream: &EventStream,
        delay: Duration,
        period: Option<Duration>,
        job: Job,
    ) -> Timer {
        let timer = Timer(Arc::default());
        {
            let mut state = self.0.state.lock().unwrap();
            if state.closed {
                timer.cancel();
                return timer;
            }

            state.seq += 1;
            let seq = state.seq;
            state.entries.push(Entry {
                due: quanta::Instant::now() + delay,
                period,
                seq,
                timer: timer.clone(),
                job,
            });

            if state.thread.is_none() {
                state.thread = Some(spawn(Arc::clone(&self.0), stream));
            }
        }

        // The new entry may be due before whatever the thread is waiting on.
        self.0.wake.notify_one();
        timer
    }

    /// Stop the thread and drop every pending timer. Waits for a job that is
    /// already running to finish, unless called from that job. Closing again
    /// does nothing.
    pub(super) fn close(&self) {
        let thread = {
            let mut state = self.0.state.lock().unwrap();
            state.closed = true;
            state.entries.clear();
            state.thread.take()
        };
        self.0.wake.notify_all();

        if let Some(thread) = thread {
            // A job that closes the stream runs on this thread: don't join yourself.
            if thread.thread().id() != thread::current().id() {
                thread.join().ok();
            }
        }
    }
}

impl Drop for Scheduler {
    /// A stream dropped without being closed would otherwise leave its thread
    /// asleep forever, since nothing would ever wake it.
    fn drop(&mut self) {
        self.close();
    }
}

fn spawn(inner: Arc<Inner>, stream: &EventStream) -> JoinHandle<()> {
    let (executor, shared) = (stream.executor(), stream.downgrade());
    thread::Builder::new()
        .name("radiate-scheduler".into())
        .spawn(move || run(inner, executor, shared))
        .expect("failed to spawn scheduler thread")
}

fn run(inner: Arc<Inner>, executor: Arc<Executor>, shared: Weak<Shared>) {
    let mut state = inner.state.lock().unwrap();

    while !state.closed {
        let now = quanta::Instant::now();
        match state.entries.peek().map(|top| top.due) {
            None => {
                state = inner.wake.wait(state).unwrap();
                continue;
            }
            Some(due) if due > now => {
                state = inner.wake.wait_timeout(state, due - now).unwrap().0;
                continue;
            }
            Some(_) => {}
        }

        let mut entry = state.entries.pop().unwrap();
        if entry.timer.cancelled() {
            continue;
        }
        drop(state);

        // Hold the stream only while the job runs, so an idle scheduler never
        // keeps it alive. If this turns out to be the last reference, dropping
        // it drops the Scheduler, whose close() marks the state closed and
        // ends the loop below.
        match EventStream::upgrade(&executor, &shared) {
            Some(stream) => {
                // A failed publish means the stream is closing; the loop
                // notices once close() marks it.
                (entry.job)(&stream).ok();
            }
            None => return,
        }

        state = inner.state.lock().unwrap();

        // Checked under the lock, so a close() that ran during the job can't
        // be undone by putting the timer back.
        if let Some(period) = entry.period
            && !state.closed
            && !entry.timer.cancelled()
        {
            // Skip missed ticks: after a stall, fire once now rather than
            // catching up with a burst. Measured from after the job, so a
            // slow job doesn't count as a missed tick.
            let now = quanta::Instant::now();
            entry.due = (entry.due + period).max(now);
            state.entries.push(entry);
        }
    }
}
