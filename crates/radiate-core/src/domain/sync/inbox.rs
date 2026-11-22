use crossbeam::channel::{self, Receiver, Sender};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct DrainResult {
    pub more: bool,
    pub processed: usize,
}

/// An unbounded FIFO queue that at most one consumer drains at a time.
///
/// Any number of threads can [`push`](Inbox::push). The push that finds the
/// inbox idle *claims* it, and the caller is then responsible for running
/// [`drain`](Inbox::drain), inline or on an executor. Every other push only
/// enqueues and returns, leaving the item for the claimed drainer.
///
/// This gives three guarantees:
/// - **One drainer at a time.** No two threads ever process items from the same
///   inbox concurrently.
/// - **FIFO.** Items pushed from one thread are drained in the order pushed.
/// - **No stranded items.** An item pushed while a drain is finishing is either
///   picked up by that drain or claims the inbox for its own pusher.
///
/// A push made from *inside* `drain`'s callback (an item processing step that
/// enqueues more work for the same inbox) loses the claim and is handled by the
/// running drain. Reentrant pushes never recurse.
pub struct Inbox<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
    scheduled: AtomicBool,
}

impl<T> Inbox<T> {
    pub fn new() -> Self {
        let (sender, receiver) = channel::unbounded();
        Inbox {
            sender,
            receiver,
            scheduled: AtomicBool::new(false),
        }
    }

    /// Enqueue `item`. Returns `true` if this call claimed the inbox. The
    /// caller must then run (or schedule) [`drain`](Inbox::drain).
    pub fn push(&self, item: T) -> bool {
        self.sender
            .send(item)
            .expect("Inbox owns its receiver, so the channel can't disconnect");
        // Pairs with the fence in `drain`. Either the drainer sees this item
        // after releasing its claim, or this load sees the release.
        std::sync::atomic::fence(Ordering::SeqCst);
        !self.scheduled.load(Ordering::Relaxed) && self.try_claim()
    }

    /// Claim the inbox without pushing. Returns `true` if this call claimed it,
    /// so the caller must then run [`drain`](Inbox::drain). Useful for holding the inbox
    /// while doing setup that may push into it, so those pushes queue up
    /// instead of starting a drain.
    pub fn try_claim(&self) -> bool {
        self.scheduled
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    /// Process queued items in order, calling `process` on each, until the
    /// inbox is empty or `budget` items have been processed.
    ///
    /// Must only be called by the holder of the claim (from a `true`
    /// [`push`](Inbox::push) or [`try_claim`](Inbox::try_claim)).
    ///
    /// Returns `true` if the budget ran out with the claim **still held**. The
    /// caller must then drain again (typically by resubmitting to an executor
    /// so other work gets a turn). Returns `false` once the claim is released.
    ///
    /// `process` must not panic. A panic would leave the inbox claimed forever,
    /// so catch panics inside `process` if it runs untrusted code.
    pub fn drain(&self, budget: usize, mut process: impl FnMut(T)) -> DrainResult {
        debug_assert!(
            self.scheduled.load(Ordering::Acquire),
            "Inbox::drain called without holding the claim"
        );

        let mut processed = 0;
        loop {
            while processed < budget {
                match self.receiver.try_recv() {
                    Ok(item) => {
                        process(item);
                        processed += 1;
                    }
                    Err(_) => break,
                }
            }

            if processed >= budget && !self.receiver.is_empty() {
                return DrainResult {
                    more: true,
                    processed,
                };
            }

            self.scheduled.store(false, Ordering::Release);

            // Pairs with the fence in `push`. A push that raced our release
            // either shows up here, or its own claim attempt succeeds.
            std::sync::atomic::fence(Ordering::SeqCst);
            if self.receiver.is_empty() || !self.try_claim() {
                return DrainResult {
                    more: false,
                    processed,
                };
            }

            if processed >= budget {
                return DrainResult {
                    more: true,
                    processed,
                };
            }
        }
    }

    /// Number of queued items not yet drained.
    pub fn len(&self) -> usize {
        self.receiver.len()
    }

    pub fn is_empty(&self) -> bool {
        self.receiver.is_empty()
    }
}

impl<T> Default for Inbox<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn first_push_claims_and_later_pushes_queue() {
        let inbox = Inbox::new();
        assert!(inbox.push(1));
        assert!(!inbox.push(2));
        assert!(!inbox.push(3));

        let mut seen = Vec::new();
        assert!(!inbox.drain(usize::MAX, |x| seen.push(x)).more);
        assert_eq!(seen, vec![1, 2, 3]);

        // Released: the next push claims again.
        assert!(inbox.push(4));
    }

    #[test]
    fn budget_keeps_the_claim_until_the_queue_is_empty() {
        let inbox = Inbox::new();
        inbox.push(0);
        for i in 1..5 {
            inbox.push(i);
        }

        let mut seen = Vec::new();
        assert!(inbox.drain(2, |x| seen.push(x)).more);
        assert!(!inbox.push(5), "claim is still held after a budgeted drain");
        assert!(inbox.drain(2, |x| seen.push(x)).more);
        assert!(!inbox.drain(usize::MAX, |x| seen.push(x)).more);
        assert_eq!(seen, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn exact_budget_on_last_item_releases_the_claim() {
        let inbox = Inbox::new();
        inbox.push(0);
        inbox.push(1);
        assert!(!inbox.drain(2, |_| {}).more);
        assert!(inbox.push(2));
    }

    #[test]
    fn reentrant_pushes_are_drained_by_the_running_drain() {
        let inbox = Inbox::new();
        assert!(inbox.push(3u32));

        let mut seen = Vec::new();
        inbox.drain(usize::MAX, |n| {
            seen.push(n);
            if n > 0 {
                assert!(!inbox.push(n - 1), "reentrant push must not claim");
            }
        });
        assert_eq!(seen, vec![3, 2, 1, 0]);
        assert!(inbox.is_empty());
    }

    #[test]
    fn try_claim_holds_the_inbox_during_setup() {
        let inbox = Inbox::new();
        assert!(inbox.try_claim());
        assert!(!inbox.push(1));
        assert!(!inbox.try_claim());

        let mut seen = Vec::new();
        assert!(!inbox.drain(usize::MAX, |x| seen.push(x)).more);
        assert_eq!(seen, vec![1]);
    }

    /// Many producers, each draining inline whenever its push claims the inbox,
    /// with a small budget to force frequent release and reclaim handoffs. Every
    /// item must be processed exactly once, and never by two drainers at once.
    #[test]
    fn concurrent_producers_never_strand_or_double_drain() {
        const THREADS: usize = 8;
        const PER_THREAD: usize = 20_000;

        let inbox = Arc::new(Inbox::new());
        let processed = Arc::new(AtomicUsize::new(0));
        let active = Arc::new(AtomicUsize::new(0));

        std::thread::scope(|scope| {
            for _ in 0..THREADS {
                let (inbox, processed, active) = (&inbox, &processed, &active);
                scope.spawn(move || {
                    for i in 0..PER_THREAD {
                        if inbox.push(i) {
                            while inbox
                                .drain(16, |_| {
                                    assert_eq!(active.fetch_add(1, Ordering::SeqCst), 0);
                                    processed.fetch_add(1, Ordering::SeqCst);
                                    active.fetch_sub(1, Ordering::SeqCst);
                                })
                                .more
                            {}
                        }
                    }
                });
            }
        });

        assert_eq!(processed.load(Ordering::SeqCst), THREADS * PER_THREAD);
        assert!(inbox.is_empty());
        assert!(inbox.try_claim(), "inbox must end released");
    }
}
