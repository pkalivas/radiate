use super::handler::EventHandler;
use super::message::Message;
use crate::{Envelope, Executor, notify::message::EventContext};
use radiate_utils::sentry_id;
use std::collections::VecDeque;
use std::fmt;
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
    mailbox: Mutex<VecDeque<(Envelope<M>, EventContext)>>,
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

    pub(super) fn mailbox_len(&self) -> usize {
        self.mailbox.lock().unwrap().len()
    }

    pub(super) fn num_processed(&self) -> u64 {
        self.num_processed.load(Ordering::Acquire)
    }

    #[inline]
    pub(super) fn tell(
        self: &Arc<Self>,
        message: Envelope<M>,
        ctx: EventContext,
        executor: &Executor,
    ) {
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

    #[inline]
    fn drain(self: Arc<Self>) {
        loop {
            let batch = std::mem::take(&mut *self.mailbox.lock().unwrap());

            if batch.is_empty() {
                self.scheduled.store(false, Ordering::Release);

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
                handler.handle(&*message, &ctx);
                self.num_processed.fetch_add(1, Ordering::AcqRel);
            }
        }
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
