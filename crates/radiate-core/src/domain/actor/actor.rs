use super::handler::{EventContext, EventHandler};
use super::message::Message;
use crate::Executor;
use radiate_utils::sentry_id;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

sentry_id!(ActorId);

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

    fn drain(self: Arc<Self>) {
        loop {
            let next = self.mailbox.lock().unwrap().pop_front();
            match next {
                Some((message, ctx)) => {
                    self.handler.lock().unwrap().handle(message, &ctx);
                    self.num_processed.fetch_add(1, Ordering::AcqRel);
                }
                None => {
                    self.scheduled.store(false, Ordering::Release);

                    // Something may have been pushed between the pop above
                    // returning `None` and clearing `scheduled`. Re-claim the
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
                }
            }
        }
    }
}

impl<M: Message> std::fmt::Debug for Actor<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Actor")
            .field("id", &self.id)
            .field("scheduled", &self.scheduled.load(Ordering::Acquire))
            .field("mailbox_size", &self.mailbox.lock().unwrap().len())
            .field("num_processed", &self.num_processed.load(Ordering::Acquire))
            .finish()
    }
}
