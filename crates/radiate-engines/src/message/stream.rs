use radiate_core::Executor;
use radiate_utils::sentry_id;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, Mutex, RwLock, atomic::AtomicU64},
};

sentry_id!(EventId);

type SubscriberMap = HashMap<TypeId, Vec<Arc<Mutex<dyn ErasedEventHandler>>>>;

pub trait Event: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Event for T {}

pub trait EventHandler<E>: Send + Sync + 'static {
    fn handle(&mut self, event: &E, ctx: &EventCtx);
}

pub trait ErasedEventHandler: Send + Sync + 'static {
    fn handle(&mut self, event: &dyn Any, ctx: &EventCtx);
}

pub struct HandleWrapper<E, H>
where
    E: Event + 'static,
    H: EventHandler<E>,
{
    handler: H,
    _marker: std::marker::PhantomData<E>,
}

impl<E, F> EventHandler<E> for F
where
    F: FnMut(&E) + Send + Sync + 'static,
{
    fn handle(&mut self, event: &E, _: &EventCtx) {
        self(event)
    }
}

impl<E, H> ErasedEventHandler for HandleWrapper<E, H>
where
    E: Event + 'static,
    H: EventHandler<E>,
{
    fn handle(&mut self, event: &dyn Any, ctx: &EventCtx) {
        if let Some(event) = event.downcast_ref::<E>() {
            self.handler.handle(event, ctx);
        }
    }
}

pub struct EventCtx {
    id: EventId,
    bus: EventStream,
}

impl EventCtx {
    pub fn id(&self) -> &EventId {
        &self.id
    }

    pub fn publish<M: Event>(&self, message: M) {
        self.bus.publish(message);
    }
}

#[derive(Clone, Default)]
pub struct EventStream {
    executor: Arc<Executor>,
    subscribers: Arc<RwLock<SubscriberMap>>,
}

impl EventStream {
    pub fn new(executor: Arc<Executor>) -> Self {
        EventStream {
            executor,
            subscribers: Arc::default(),
        }
    }

    pub fn set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor;
    }

    pub fn subscribe<M: Event>(&self, handler: impl EventHandler<M>) {
        let type_id = TypeId::of::<M>();
        let handler = HandleWrapper {
            handler,
            _marker: std::marker::PhantomData,
        };

        self.subscribers
            .write()
            .unwrap()
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push(Arc::new(Mutex::new(handler)));
    }

    pub fn lazy_publish<M: Event>(&self, f: impl FnOnce() -> M) {
        if self.can_publish::<M>() {
            self.publish(f());
        }
    }

    pub fn publish<M: Event>(&self, message: M) {
        let type_id = TypeId::of::<M>();
        if let Some(subscribers) = self.subscribers.read().unwrap().get(&type_id) {
            let arc_msg = Arc::new(message);
            let id = EventId::new();

            for sub in subscribers.iter() {
                let cloned_subs = Arc::clone(sub);
                let message = Arc::clone(&arc_msg);
                let ctx = EventCtx {
                    id,
                    bus: self.clone(),
                };

                self.executor.submit(move || {
                    let mut subs = cloned_subs.lock().unwrap();
                    subs.handle(message.as_ref(), &ctx);
                });
            }
        }
    }

    fn can_publish<M: Event>(&self) -> bool {
        self.subscribers
            .read()
            .unwrap()
            .contains_key(&TypeId::of::<M>())
    }
}

impl Debug for EventStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let subscribers = self.subscribers.read().unwrap();
        write!(
            f,
            "EventStream(subscribers={}, executor={:?})",
            subscribers.len(),
            self.executor
        )
    }
}
