use radiate_core::Executor;
use radiate_utils::sentry_id;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, Mutex, RwLock, atomic::AtomicU64},
};

sentry_id!(EventId);

type Subscriber<E> = Box<dyn EventHandler<E>>;
type SubscriberMap = HashMap<TypeId, Vec<Arc<Mutex<dyn Any + Send + Sync>>>>;

pub trait Event: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Event for T {}

pub trait EventHandler<E>: Send + Sync + 'static {
    fn handle(&mut self, event: &E, ctx: &EventCtx);
}

impl<E, F> EventHandler<E> for F
where
    F: FnMut(&E) + Send + Sync + 'static,
{
    fn handle(&mut self, event: &E, _: &EventCtx) {
        self(event)
    }
}

pub struct EventCtx {
    id: EventId,
    bus: EventBus,
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
pub struct EventBus {
    executor: Arc<Executor>,
    subscribers: Arc<RwLock<SubscriberMap>>,
}

impl EventBus {
    pub fn new(executor: Arc<Executor>) -> Self {
        EventBus {
            executor,
            subscribers: Arc::default(),
        }
    }

    pub fn set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor;
    }

    pub fn subscribe<M: Event>(&self, handler: impl EventHandler<M>) {
        let subscriber: Subscriber<M> = Box::new(handler);
        let type_id = TypeId::of::<M>();

        self.subscribers
            .write()
            .unwrap()
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push(Arc::new(Mutex::new(subscriber)));
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

            for sub in subscribers.iter() {
                let cloned_subs = Arc::clone(sub);
                let message = Arc::clone(&arc_msg);
                let ctx = EventCtx {
                    id: EventId::new(),
                    bus: self.clone(),
                };

                self.executor.submit(move || {
                    let mut subs = cloned_subs.lock().unwrap();
                    if let Some(handler) = subs.downcast_mut::<Subscriber<M>>() {
                        handler.handle(&message, &ctx);
                    }
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
