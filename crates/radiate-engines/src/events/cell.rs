use crate::{
    Actor,
    events::{
        Addr,
        addr::{ActorContext, Envelope},
    },
};
use crossbeam::channel::{self, Receiver, Sender};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

pub(super) struct Mailbox<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
    scheduled: AtomicBool,
}

impl<T> Mailbox<T> {
    fn new() -> Self {
        let (sender, receiver) = channel::unbounded();
        Mailbox {
            sender,
            receiver,
            scheduled: AtomicBool::new(false),
        }
    }

    fn try_claim(&self) -> bool {
        self.scheduled
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    #[inline]
    fn drain(&self, mut process: impl FnMut(T)) {
        loop {
            while let Ok(item) = self.receiver.try_recv() {
                process(item);
            }

            self.scheduled.store(false, Ordering::Release);

            if !self.try_claim() {
                break;
            }

            let Some(item) = self.receiver.try_recv().ok() else {
                self.scheduled.store(false, Ordering::Release);
                break;
            };

            process(item);
        }
    }
}

pub struct ActorCell<A> {
    pub(super) actor: Mutex<A>,
    pub(super) mailbox: Mailbox<Envelope<A>>,
    pub(super) alive: AtomicBool,
}

impl<A: Actor> ActorCell<A> {
    pub(super) fn new(actor: A) -> Self {
        ActorCell {
            actor: Mutex::new(actor),
            mailbox: Mailbox::new(),
            alive: AtomicBool::new(true),
        }
    }

    pub(super) fn try_claim(&self) -> bool {
        self.mailbox.try_claim()
    }

    pub(super) fn enqueue(&self, envelope: Envelope<A>) -> bool {
        if !self.alive.load(Ordering::Acquire) {
            return false;
        }

        self.mailbox.sender.send(envelope).is_ok()
    }

    pub(super) fn process_batch(self: Arc<Self>, addr: Addr<A>) {
        let mut actor = self.actor.lock().unwrap_or_else(|e| e.into_inner());
        let mut stopping = false;

        self.mailbox.drain(|envelope| {
            if stopping {
                return;
            }

            let ctx = ActorContext(addr.clone());
            let result = catch_unwind(AssertUnwindSafe(|| (envelope.run)(&mut actor, &ctx)));

            if result.is_err() {
                self.alive.store(false, Ordering::Release);
                stopping = true;
            }
        });
    }
}
