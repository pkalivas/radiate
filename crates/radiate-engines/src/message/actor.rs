use crate::{
    EventCtx, EventHandler,
    message::{EventStream, MailboxId},
};
use crossbeam::channel::{self, Receiver, Sender};
use radiate_core::Executor;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

pub trait Event: Message<Response = ()> + std::fmt::Debug {}
impl<T> Event for T where T: Message<Response = ()> + std::fmt::Debug {}

pub trait Message: Send + Sync + 'static {
    type Response: Send + 'static;
}

impl<T: Send + Sync + 'static> Message for Arc<T> {
    type Response = ();
}

pub trait Handler<M>: Actor
where
    M: Message,
{
    fn handle(&mut self, msg: M, ctx: &ActorContext<Self>) -> M::Response
    where
        Self: Sized;
}

struct Envelope<A> {
    run: Box<dyn FnOnce(&mut A, &ActorContext<A>) + Send>,
    span: tracing::Span,
}

pub trait Actor: Send + Sync + 'static {
    fn started(&mut self, ctx: &ActorContext<Self>)
    where
        Self: Sized,
    {
    }
}

pub trait Scheduler<I>: Send + Sync + 'static {
    fn schedule(&self, item: I);
}

pub struct ActorContext<A>(Addr<A>, tracing::Span);

impl<A: Actor> ActorContext<A> {
    pub fn send<M>(&self, msg: M)
    where
        A: Handler<M>,
        M: Message,
    {
        (self.0).send(msg);
    }

    pub fn ask<M>(&self, msg: M) -> M::Response
    where
        A: Handler<M>,
        M: Message,
    {
        (self.0).ask(msg)
    }

    pub fn publish<E: Event>(&self, message: E) {
        (self.0).publish(message);
    }
}

struct Mailbox<T> {
    id: MailboxId,
    sender: Sender<T>,
    receiver: Receiver<T>,
    scheduled: AtomicBool,
}

impl<T> Mailbox<T> {
    fn new() -> Self {
        let (sender, receiver) = channel::unbounded();
        Mailbox {
            id: MailboxId::new(),
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
        let mut processed = 0;
        loop {
            while let Ok(item) = self.receiver.try_recv() {
                process(item);
                processed += 1;
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
            processed += 1;
        }
        if processed > 0 {
            tracing::info!(mailbox = %self.id, processed, thread = ?std::thread::current().id(), "processed batch");
        }
    }
}

pub struct ActorCell<A> {
    actor: Mutex<A>,
    mailbox: Mailbox<Envelope<A>>,
}

impl<A: Actor> ActorCell<A> {
    fn new(actor: A) -> Self {
        ActorCell {
            actor: Mutex::new(actor),
            mailbox: Mailbox::new(),
        }
    }

    fn enqueue(&self, envelope: Envelope<A>) {
        self.mailbox.sender.send(envelope).unwrap();
    }

    fn process_batch(self: Arc<Self>, addr: Addr<A>) {
        let mut actor = self.actor.lock().unwrap_or_else(|e| e.into_inner());
        self.mailbox.drain(|envelope| {
            let _guard = envelope.span.enter();
            let ctx = ActorContext(addr.clone(), envelope.span.clone());

            (envelope.run)(&mut actor, &ctx);
        });
    }
}

impl<A, F> Scheduler<F> for ActorCell<A>
where
    A: Actor,
    F: FnOnce(&mut A, &ActorContext<A>) + Send + 'static,
{
    fn schedule(&self, item: F) {
        self.enqueue(Envelope {
            run: Box::new(item),
            span: tracing::info_span!("actor", ty = %std::any::type_name::<A>()),
        });
    }
}

pub struct Addr<A> {
    cell: Arc<ActorCell<A>>,
    executor: Arc<Executor>,
    bus: Option<EventStream>,
}

impl<A> Clone for Addr<A> {
    fn clone(&self) -> Self {
        Addr {
            cell: Arc::clone(&self.cell),
            executor: Arc::clone(&self.executor),
            bus: self.bus.clone(),
        }
    }
}

impl<A: Actor> Addr<A> {
    pub fn spawn(actor: A, executor: Arc<Executor>) -> Self {
        Self::spawn_with_bus(actor, executor, None)
    }

    pub fn spawn_with_bus(actor: A, executor: Arc<Executor>, bus: Option<EventStream>) -> Self {
        let cell = Arc::new(ActorCell::new(actor));
        let addr = Addr {
            cell,
            executor,
            bus,
        };

        {
            let mut guard = addr.cell.actor.lock().unwrap_or_else(|e| e.into_inner());
            let span = tracing::info_span!("actor", ty = %std::any::type_name::<A>());
            let ctx = ActorContext(addr.clone(), span);
            guard.started(&ctx);
        }

        addr
    }

    pub fn send_traced<M>(&self, msg: M, span: tracing::Span)
    where
        A: Handler<M>,
        M: Message,
    {
        self.cell.enqueue(Envelope {
            run: Box::new(move |actor: &mut A, ctx: &ActorContext<A>| {
                actor.handle(msg, ctx);
            }),
            span,
        });
        self.dispatch();
    }

    pub fn send<M>(&self, msg: M)
    where
        A: Handler<M>,
        M: Message,
    {
        self.cell
            .schedule(move |actor: &mut A, ctx: &ActorContext<A>| {
                actor.handle(msg, ctx);
            });
        self.dispatch();
    }

    pub fn ask<M>(&self, msg: M) -> M::Response
    where
        A: Handler<M>,
        M: Message,
    {
        let (tx, rx) = channel::bounded(1);

        self.cell
            .schedule(move |actor: &mut A, ctx: &ActorContext<A>| {
                let response = actor.handle(msg, ctx);
                tx.send(response).unwrap();
            });
        self.dispatch();

        rx.recv().unwrap()
    }

    pub fn publish<E: Event>(&self, message: E) {
        if let Some(bus) = &self.bus {
            bus.publish(Arc::new(message));
        }
    }

    pub fn subscribe<E: Event>(&self)
    where
        A: Handler<Arc<E>>,
    {
        if let Some(bus) = &self.bus {
            bus.subscribe::<Arc<E>>(self.clone());
        }
    }

    fn dispatch(&self) {
        if self.cell.mailbox.try_claim() {
            let cell = Arc::clone(&self.cell);
            let addr = self.clone();
            self.executor.submit(move || cell.process_batch(addr));
        }
    }
}

impl<A, E> EventHandler<Arc<E>> for Addr<A>
where
    A: Handler<Arc<E>>,
    E: Event,
{
    fn handle(&mut self, message: &Arc<E>, ctx: &EventCtx) {
        self.send_traced(Arc::clone(message), ctx.span().clone());
    }
}
