use super::handler::{EventContext, EventHandler};
use super::message::Message;
use crate::Executor;
use radiate_utils::sentry_id;
use std::any::Any;
use std::collections::VecDeque;
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

sentry_id!(ActorId);

/// Type-erased handle to an `Actor<M>` for a caller that no longer has `M`
/// in scope (`ActorSystem`'s registry holds actors for many different
/// message types side by side). `dyn Any` alone gets you back to the
/// concrete type via `downcast`, but its own `Debug` impl only ever prints
/// an opaque placeholder — it has no way to reach `Actor<M>`'s real one.
/// This trait is the fix: `as_any_arc` recovers the concrete `Arc<Actor<M>>`
/// for dispatch, `debug_actor` recovers real `Debug` output through the same
/// erasure.
pub(super) trait AnyActor: Send + Sync {
    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
    fn debug_actor(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
    fn num_processed(&self) -> u64;
    fn mailbox_len(&self) -> usize;
}

impl fmt::Debug for dyn AnyActor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug_actor(f)
    }
}

/// A single subscriber's mailbox. `tell` enqueues (message, context) pairs
/// and, if nobody is currently draining this actor, schedules a drain on the
/// executor. `scheduled` guarantees at most one in-flight drain per actor,
/// which is what gives every actor FIFO delivery and non-concurrent handling
/// regardless of how many worker threads the executor itself has.
///
/// The context is captured per-message at `tell` time (same as the
/// executor), not looked up fresh at drain time — `ActorSystem::set_sync`
/// only ever happens once, early, before real traffic starts, so this is
/// just the simplest thing that's still correct.
pub(super) struct Actor<M: Message> {
    id: ActorId,
    handler: Mutex<Box<dyn EventHandler<M>>>,
    mailbox: Mutex<VecDeque<(M, EventContext)>>,
    scheduled: AtomicBool,
    num_processed: AtomicU64,
}

impl<M: Message> Actor<M> {
    pub(super) fn new(handler: Box<dyn EventHandler<M>>) -> Arc<Self> {
        Arc::new(Actor {
            id: ActorId::new(),
            handler: Mutex::new(handler),
            mailbox: Mutex::new(VecDeque::new()),
            scheduled: AtomicBool::new(false),
            num_processed: AtomicU64::new(0),
        })
    }

    pub(super) fn tell(self: &Arc<Self>, message: M, ctx: EventContext, executor: &Executor) {
        self.mailbox.lock().unwrap().push_back((message, ctx));

        if self
            .scheduled
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            let this = Arc::clone(self);
            executor.submit(move || this.drain());
        }
    }

    /// Drains in batches rather than one `pop_front` per message: each pass
    /// swaps the *entire* current mailbox out under a single lock, then
    /// processes it without holding that lock at all. Locking per-message
    /// instead would mean every message fights concurrent `tell` calls (from
    /// any producer thread) for the same `Mutex` — under load that
    /// contention dominates, since this is the one lock every producer and
    /// the drain itself both need.
    fn drain(self: Arc<Self>) {
        loop {
            let batch = std::mem::take(&mut *self.mailbox.lock().unwrap());

            if batch.is_empty() {
                self.scheduled.store(false, Ordering::Release);

                // Something may have been pushed between the take above
                // returning empty and clearing `scheduled`. Re-claim the
                // slot and keep draining if so, otherwise we're done.
                let more_arrived = !self.mailbox.lock().unwrap().is_empty();
                if !more_arrived
                    || self
                        .scheduled
                        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                        .is_err()
                {
                    return;
                }
                continue;
            }

            let mut handler = self.handler.lock().unwrap();
            for (message, ctx) in batch {
                handler.handle(message, &ctx);
                self.num_processed.fetch_add(1, Ordering::AcqRel);
            }
        }
    }
}

impl<M: Message> AnyActor for Actor<M> {
    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }

    fn debug_actor(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }

    fn num_processed(&self) -> u64 {
        self.num_processed.load(Ordering::Acquire)
    }

    fn mailbox_len(&self) -> usize {
        self.mailbox.lock().unwrap().len()
    }
}

impl<M: Message> fmt::Debug for Actor<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Actor")
            .field("id", &self.id)
            .field("message_type", &std::any::type_name::<M>())
            .field("scheduled", &self.scheduled.load(Ordering::Acquire))
            .field("mailbox_size", &self.mailbox.lock().unwrap().len())
            .field("num_processed", &self.num_processed.load(Ordering::Acquire))
            .finish()
    }
}
