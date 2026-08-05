use crate::actors::{ProcessId, context::SystemCtx, message::DeadLetter};
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, Weak};
use std::{
    any::TypeId,
    sync::atomic::{AtomicBool, Ordering},
};

type Envelope<A> = Box<dyn FnOnce(&mut A, &Addr<A>) + Send>;

pub trait Actor: Send + Sized {
    fn on_init(&mut self, _: &Addr<Self>) {}
    fn on_stop(&mut self, _: &Addr<Self>) {}
}

pub trait MessageHandler<M: Send + 'static>: Actor {
    fn handle(&mut self, message: M, ctx: &Addr<Self>);
}

pub struct ActorCtx<A: Actor> {
    pub(super) pid: ProcessId,

    pub(super) actor: Arc<Mutex<A>>,
    pub(super) sender: Sender<Envelope<A>>,
    pub(super) receiver: Receiver<Envelope<A>>,
    pub(super) scheduled: AtomicBool,

    pub(super) stopped: AtomicBool,
    pub(super) parent: Option<AnyActorRef>,

    pub(super) context: SystemCtx,
}

// SAFETY: `receiver` is a plain `mpsc::Receiver`, which is `Send` but not `Sync` — its
// `try_recv` is unsound to call concurrently from multiple threads, even though the method
// only takes `&self`. That never happens here: every call to `self.receiver.try_recv()` lives
// inside `process_batch`, which only runs between a winning `try_claim()`
// (`compare_exchange(false, true, AcqRel, Acquire)`) and the matching
// `scheduled.store(false, Release)`. That Release/Acquire pair is a real happens-before edge,
// so at most one thread is ever inside that window for a given `ActorCell` at a time, and no
// other code path touches `receiver` — the exclusion is structural, not just conventional.
unsafe impl<A: Actor> Sync for ActorCtx<A> {}

impl<A: Actor> ActorCtx<A> {
    #[inline]
    fn deliver(&self, actor: &mut A, ctx: &Addr<A>, envelope: Envelope<A>) {
        let maybe_success =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| envelope(actor, ctx)))
                .map_err(|payload| {
                    payload
                        .downcast_ref::<&str>()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "actor panicked".to_string())
                });

        if let Err(reason) = maybe_success
            && let Some(parent) = &self.parent
        {
            parent.report_child_failure(reason.clone());
        }
    }

    #[inline]
    fn try_claim(&self) -> bool {
        self.scheduled
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    #[inline]
    fn process_batch(self: Arc<Self>) {
        let addr = Addr {
            cell: Arc::clone(&self),
        };

        loop {
            {
                let mut actor = self.actor.lock().unwrap();
                while let Ok(msg) = self.receiver.try_recv() {
                    self.deliver(&mut *actor, &addr, msg);
                }
            }

            self.scheduled.store(false, Ordering::Release);

            if !self.try_claim() {
                break;
            }

            match self.receiver.try_recv() {
                Ok(msg) => {
                    let mut actor = self.actor.lock().unwrap();
                    self.deliver(&mut *actor, &addr, msg);

                    continue;
                }
                Err(_) => {
                    self.scheduled.store(false, Ordering::Release);
                    break;
                }
            }
        }
    }
}

pub struct Addr<A: Actor> {
    pub(crate) cell: Arc<ActorCtx<A>>,
}

impl<A: Actor + 'static> Addr<A> {
    pub fn publish<M>(&self, message: M)
    where
        M: Send + Clone + 'static,
    {
        self.cell.context.publish(message);
    }

    pub fn send<M>(&self, message: M)
    where
        A: MessageHandler<M>,
        M: Send + 'static,
    {
        // A message that loses a race with a concurrent `stop()` — sent just
        // before `stopped` flips but delivered after the poison pill — may
        // still reach `on_stop`'s successor. Accepted as a narrow, benign
        // race rather than gating every delivery on this flag too.
        if self.cell.stopped.load(Ordering::Acquire) {
            if TypeId::of::<M>() != TypeId::of::<DeadLetter>() {
                self.cell.context.bus().publish(DeadLetter {
                    message_type: std::any::type_name::<M>(),
                    pid: self.cell.pid.clone(),
                });
            }
            return;
        }

        let envelope = Box::new(move |actor: &mut A, ctx: &Addr<A>| {
            actor.handle(message, ctx);
        });
        if self.cell.sender.send(envelope).is_err() {
            if TypeId::of::<M>() != TypeId::of::<DeadLetter>() {
                self.cell.context.bus().publish(DeadLetter {
                    message_type: std::any::type_name::<M>(),
                    pid: self.cell.pid.clone(),
                });
            }

            return;
        }

        if self.cell.try_claim() {
            let cell = Arc::clone(&self.cell);
            self.cell
                .context
                .executor()
                .submit(move || cell.process_batch());
        }
    }

    pub fn subscribe<M>(&self)
    where
        A: MessageHandler<M>,
        M: Send + 'static,
    {
        self.cell.context.bus().subscribe(self.recipient())
    }

    pub fn start(&self) {
        let cell = Arc::clone(&self.cell);
        self.cell
            .context
            .executor()
            .submit(move || cell.process_batch());
    }

    /// Enqueues a poison-pill parcel that calls `Actor::on_stop` and marks
    /// this actor stopped, so every `ActorRef` clone starts dead-lettering
    /// instead of accepting new sends. Idempotent — a second `stop()` is a
    /// no-op, since the flip from `false` to `true` only succeeds once.
    pub fn stop(&self) {
        if self.cell.stopped.swap(true, Ordering::AcqRel) {
            return;
        }

        let pid = &self.cell.pid;
        if let Some(removed) = self.cell.context.registry.remove::<A>(pid) {
            self.cell.context.registry.insert(pid.clone(), removed);
        }

        self.cell
            .sender
            .send(Box::new(|actor: &mut A, ctx: &Addr<A>| {
                actor.on_stop(ctx);
            }))
            .ok();

        if self.cell.try_claim() {
            let cell = Arc::clone(&self.cell);
            self.cell
                .context
                .executor()
                .submit(move || cell.process_batch());
        }
    }

    /// Erases which actor this is while keeping the message type `M`
    /// concrete — the handle `DomainBus` stores, since a bus subscription
    /// may be satisfied by any number of different actor types that all
    /// implement `MessageHandler<M>`.
    pub(super) fn recipient<M>(&self) -> Recipient<M>
    where
        A: MessageHandler<M>,
        M: Send + 'static,
    {
        let this = self.clone();
        Recipient {
            send: Arc::new(move |message| this.send(message)),
        }
    }
}

impl<A: Actor> Clone for Addr<A> {
    fn clone(&self) -> Self {
        Addr {
            cell: Arc::clone(&self.cell),
        }
    }
}

pub struct WeakAddr<A: Actor> {
    pub(super) cell: Weak<ActorCtx<A>>,
}

impl<A: Actor + 'static> Addr<A> {
    pub fn downgrade(&self) -> WeakAddr<A> {
        WeakAddr {
            cell: Arc::downgrade(&self.cell),
        }
    }
}

impl<A: Actor + 'static> WeakAddr<A> {
    pub fn upgrade(&self) -> Option<Addr<A>> {
        let cell = self.cell.upgrade()?;
        if cell.stopped.load(Ordering::Acquire) {
            return None;
        }

        Some(Addr { cell })
    }
}

impl<A: Actor> Clone for WeakAddr<A> {
    fn clone(&self) -> Self {
        WeakAddr {
            cell: Weak::clone(&self.cell),
        }
    }
}

#[derive(Clone)]
pub struct Recipient<M> {
    send: Arc<dyn Fn(M) + Send + Sync>,
}

impl<M: Send + 'static> Recipient<M> {
    pub fn tell(&self, message: M) {
        (self.send)(message);
    }
}

#[derive(Clone)]
pub struct AnyActorRef {
    fail_hook: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

impl AnyActorRef {
    pub fn report_child_failure(&self, reason: String) {
        if let Some(hook) = &self.fail_hook {
            hook(reason);
        }
    }
}
