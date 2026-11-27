use crate::{Flag, FlagReader};
use std::{
    cmp::Ordering as Cmp,
    collections::BinaryHeap,
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

/// What a job wants once it has run. Only matters for periodic jobs: a
/// one-shot job runs once whatever it returns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Next {
    /// Run again after the period.
    Continue,
    /// Retire the timer, for example once whatever the job feeds has gone away.
    Stop,
}

pub type Job = Box<dyn FnMut() -> Next + Send>;

/// Handle to a scheduled job. Holding a `Timer` lets you cancel the job;
/// [`reader`](Self::reader) lends out a read-only view of that cancellation
/// (e.g. to the job itself, or to whatever it feeds) without the power to cancel.
#[derive(Clone, Debug, Default)]
pub struct Timer(Flag);

impl Timer {
    /// Stop the job from running again. A run already in progress finishes.
    pub fn cancel(&self) {
        self.0.raise();
    }

    /// Cancelled, or refused because the scheduler was already closed.
    pub fn is_cancelled(&self) -> bool {
        self.0.is_raised()
    }

    /// Read-only cancellation signal.
    pub fn reader(&self) -> FlagReader {
        self.0.reader()
    }
}

struct Entry {
    due: quanta::Instant,
    period: Option<Duration>,
    seq: u64,
    cancelled: FlagReader,
    job: Job,
}

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
    closed: bool,
    seq: u64,
    thread: Option<JoinHandle<()>>,
}

#[derive(Default)]
struct Inner {
    state: Mutex<State>,
    wake: Condvar,
}

/// Runs jobs after a delay, or repeatedly at a fixed period, on one
/// background thread.
///
/// The thread starts with the first job, so a scheduler that never schedules
/// anything costs nothing. Open from creation until [`close`](Self::close);
/// dropping it closes it.
///
/// A periodic job that falls behind (a slow job, or a stall) runs once when
/// it can rather than catching up with a burst of missed runs.
#[derive(Default)]
pub struct Scheduler(Arc<Inner>);

impl Scheduler {
    pub fn after(&self, delay: Duration, job: impl FnOnce() + Send + 'static) -> Timer {
        let mut job = Some(job);
        self.schedule(
            delay,
            None,
            Box::new(move || {
                if let Some(job) = job.take() {
                    job();
                }
                Next::Stop
            }),
        )
    }

    pub fn every(&self, period: Duration, job: impl FnMut() -> Next + Send + 'static) -> Timer {
        self.schedule(period, Some(period), Box::new(job))
    }

    fn schedule(&self, delay: Duration, period: Option<Duration>, job: Job) -> Timer {
        let timer = Timer::default();
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
                cancelled: timer.reader(),
                job,
            });

            if state.thread.is_none() {
                state.thread = Some(spawn(Arc::clone(&self.0)));
            }
        }

        // The new entry may be due before whatever the thread is waiting on.
        self.0.wake.notify_one();
        timer
    }

    /// Stop the thread and drop every pending job. Waits for a job that is
    /// already running to finish, unless called from that job. Closing again
    /// does nothing.
    pub fn close(&self) {
        let thread = {
            let mut state = self.0.state.lock().unwrap();
            state.closed = true;
            state.entries.clear();
            state.thread.take()
        };
        self.0.wake.notify_all();

        if let Some(thread) = thread {
            // A job that closes its own scheduler runs on this thread: don't
            // join yourself.
            if thread.thread().id() != thread::current().id() {
                thread.join().ok();
            }
        }
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        self.close();
    }
}

fn spawn(inner: Arc<Inner>) -> JoinHandle<()> {
    thread::Builder::new()
        .name("radiate-scheduler".into())
        .spawn(move || run(inner))
        .expect("failed to spawn scheduler thread")
}

fn run(inner: Arc<Inner>) {
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
        if entry.cancelled.is_raised() {
            continue;
        }
        drop(state);

        let next = (entry.job)();

        state = inner.state.lock().unwrap();

        // Checked under the lock, so a close() that ran during the job can't
        // be undone by putting the job back.
        if let (Some(period), Next::Continue) = (entry.period, next)
            && !state.closed
            && !entry.cancelled.is_raised()
        {
            // Skip missed runs: after a stall, run once now rather than
            // catching up with a burst. Measured from after the job, so a
            // slow job doesn't count as a missed run.
            entry.due = (entry.due + period).max(quanta::Instant::now());
            state.entries.push(entry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    // Long enough that a passing run never hits it; only a hang does.
    const GENEROUS: Duration = Duration::from_secs(5);

    #[test]
    fn a_job_runs_once_after_its_delay() {
        let scheduler = Scheduler::default();
        let (tx, rx) = mpsc::channel();
        scheduler.after(Duration::from_millis(1), move || tx.send(()).unwrap());
        rx.recv_timeout(GENEROUS).expect("ran");
    }

    #[test]
    fn jobs_due_together_run_in_the_order_they_were_scheduled() {
        let scheduler = Scheduler::default();
        let (tx, rx) = mpsc::channel();
        for i in 0..5 {
            let tx = tx.clone();
            scheduler.after(Duration::ZERO, move || tx.send(i).unwrap());
        }
        let order = (0..5)
            .map(|_| rx.recv_timeout(GENEROUS).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(order, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn a_periodic_job_repeats_until_it_stops_itself() {
        let scheduler = Scheduler::default();
        let (tx, rx) = mpsc::channel();
        let mut runs = 0;
        scheduler.every(Duration::from_millis(1), move || {
            runs += 1;
            tx.send(runs).unwrap();
            if runs == 3 {
                Next::Stop
            } else {
                Next::Continue
            }
        });

        assert_eq!(rx.recv_timeout(GENEROUS), Ok(1));
        assert_eq!(rx.recv_timeout(GENEROUS), Ok(2));
        assert_eq!(rx.recv_timeout(GENEROUS), Ok(3));

        // It's retired, so the channel's sender was dropped with it.
        assert!(rx.recv_timeout(GENEROUS).is_err());
    }

    #[test]
    fn a_cancelled_job_never_runs() {
        let scheduler = Scheduler::default();
        let (tx, rx) = mpsc::channel();
        let cancelled = scheduler.after(Duration::from_millis(1), {
            let tx = tx.clone();
            move || tx.send("cancelled").unwrap()
        });
        cancelled.cancel();
        scheduler.after(Duration::from_millis(2), move || tx.send("marker").unwrap());

        assert_eq!(rx.recv_timeout(GENEROUS), Ok("marker"));
    }

    #[test]
    fn a_closed_scheduler_refuses_new_jobs() {
        let scheduler = Scheduler::default();
        scheduler.close();
        let timer = scheduler.after(Duration::ZERO, || panic!("ran after close"));
        assert!(timer.is_cancelled());
    }

    #[test]
    fn dropping_the_scheduler_stops_its_thread() {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let scheduler = Scheduler::default();
            scheduler.every(Duration::from_secs(3600), || Next::Continue);
            drop(scheduler); // would hang if the thread were never woken
            tx.send(()).unwrap();
        });
        rx.recv_timeout(GENEROUS).expect("drop returned");
    }

    #[test]
    fn a_lent_reader_sees_cancellation() {
        let scheduler = Scheduler::default();
        let timer = scheduler.every(Duration::from_secs(3600), || Next::Continue);
        let reader = timer.reader();
        assert!(!reader.is_raised());
        timer.cancel();
        assert!(reader.is_raised());
    }
}
