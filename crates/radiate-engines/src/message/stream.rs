use crate::{
    Actor,
    message::actor::{ActorContext, Addr, MessageHandler},
};
use radiate_core::Executor;
use radiate_utils::sentry_id;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    marker::PhantomData,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

sentry_id!(EventId);

pub trait Event: Send + Sync + 'static {
    fn event_label() -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl<T: Send + Sync + 'static> Event for T {}

pub trait EventHandler<E>: Send + Sync + 'static {
    fn handle(&mut self, event: &E);
}

impl<E, F> EventHandler<E> for F
where
    E: Event,
    F: FnMut(&E) + Send + Sync + 'static,
{
    fn handle(&mut self, event: &E) {
        self(event)
    }
}

pub struct Subscription {
    active: Arc<AtomicBool>,
}

impl Subscription {
    pub fn unsubscribe(&self) {
        self.active.store(false, Ordering::Release);
    }
}

type Payload = Arc<dyn Any + Send + Sync>;
type Forward = Arc<dyn Fn(Payload, tracing::Span) + Send + Sync>;

#[derive(Clone)]
struct Registration {
    forward: Forward,
    active: Arc<AtomicBool>,
}

type SubscriberList = Arc<Vec<Registration>>;
type SubscriberMap = HashMap<TypeId, SubscriberList>;

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

    pub fn spawn<A: Actor>(&self, actor: A) -> Addr<A> {
        Addr::spawn_with_bus(actor, Arc::clone(&self.executor), Some(self.clone()))
    }

    pub fn set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor;
    }

    pub fn subscribe_addr<E, A>(&self, addr: &Addr<A>) -> Subscription
    where
        E: Event,
        A: MessageHandler<Arc<E>>,
    {
        let active = Arc::new(AtomicBool::new(true));
        let addr = addr.clone();

        let forward = Arc::new(move |payload: Payload, span: tracing::Span| {
            if let Ok(msg) = payload.downcast::<E>() {
                addr.send_traced(msg, span);
            }
        });

        self.register(
            TypeId::of::<E>(),
            Registration {
                forward,
                active: Arc::clone(&active),
            },
        );

        Subscription { active }
    }

    pub fn subscribe<E: Event>(&self, handler: impl EventHandler<E>) -> Subscription {
        let addr = Addr::spawn(
            HandlerActor {
                handler,
                _marker: PhantomData,
            },
            Arc::clone(&self.executor),
        );
        self.subscribe_addr::<E, _>(&addr)
    }

    pub fn publish<E: Event>(&self, message: E) {
        let id = EventId::new();
        let span = tracing::info_span!("event", ty = %E::event_label(), %id);
        let _enter = span.enter();

        let group = {
            let subscribers = self.subscribers.read().unwrap();
            match subscribers.get(&TypeId::of::<E>()) {
                Some(group) => Arc::clone(group),
                None => return,
            }
        };

        let payload: Payload = Arc::new(message);

        for registration in group.iter() {
            if !registration.active.load(Ordering::Acquire) {
                continue;
            }

            (registration.forward)(Arc::clone(&payload), span.clone());
        }
    }

    pub fn lazy_publish<E: Event>(&self, f: impl FnOnce() -> E) {
        if self.can_publish::<E>() {
            self.publish(f());
        }
    }

    pub fn handler_count<E: Event>(&self) -> usize {
        self.subscribers
            .read()
            .unwrap()
            .get(&TypeId::of::<E>())
            .map(|g| g.len())
            .unwrap_or(0)
    }

    fn register(&self, type_id: TypeId, registration: Registration) {
        let mut subscribers = self.subscribers.write().unwrap();
        let list = subscribers.entry(type_id).or_insert_with(Arc::default);
        Arc::make_mut(list).push(registration);
    }

    fn can_publish<E: Event>(&self) -> bool {
        self.subscribers
            .read()
            .unwrap()
            .contains_key(&TypeId::of::<E>())
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

struct HandlerActor<M, H> {
    handler: H,
    _marker: PhantomData<fn(&M)>,
}

impl<M, H> Actor for HandlerActor<M, H>
where
    M: Event,
    H: EventHandler<M>,
{
}

impl<M, H> MessageHandler<Arc<M>> for HandlerActor<M, H>
where
    M: Event,
    H: EventHandler<M>,
{
    fn handle(&mut self, msg: Arc<M>, _ctx: &ActorContext<Self>) {
        self.handler.handle(&msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    #[derive(Debug, Clone, PartialEq)]
    struct Ping(u32);

    fn throughput(count: usize, elapsed: Duration) -> f64 {
        count as f64 / elapsed.as_secs_f64()
    }

    #[test]
    #[ignore = "throughput smoke test: cargo test --release -- --ignored --nocapture"]
    fn throughput_publish_single_subscriber_serial() {
        const N: usize = 500_000;
        let bus = EventStream::new(Arc::new(Executor::Serial));
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);

        bus.subscribe::<Ping>(move |_: &Ping| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        let start = Instant::now();
        for i in 0..N {
            bus.publish(Ping(i as u32));
        }
        let elapsed = start.elapsed();

        assert_eq!(count.load(Ordering::SeqCst), N);
        println!(
            "[publish/1 sub]     {N} events in {elapsed:?} = {:.0} events/sec",
            throughput(N, elapsed)
        );
    }

    #[test]
    #[ignore = "throughput smoke test: cargo test --release -- --ignored --nocapture"]
    fn throughput_publish_fanout_serial() {
        const N: usize = 100_000;
        const SUBSCRIBERS: usize = 8;

        let bus = EventStream::new(Arc::new(Executor::Serial));
        let counts: Vec<_> = (0..SUBSCRIBERS)
            .map(|_| Arc::new(AtomicUsize::new(0)))
            .collect();

        for count in &counts {
            let count = Arc::clone(count);
            bus.subscribe::<Ping>(move |_: &Ping| {
                count.fetch_add(1, Ordering::SeqCst);
            });
        }

        let start = Instant::now();
        for i in 0..N {
            bus.publish(Ping(i as u32));
        }
        let elapsed = start.elapsed();

        for count in &counts {
            assert_eq!(count.load(Ordering::SeqCst), N);
        }

        let deliveries = N * SUBSCRIBERS;
        println!(
            "[publish/fanout x{SUBSCRIBERS}] {deliveries} deliveries in {elapsed:?} = {:.0} deliveries/sec",
            throughput(deliveries, elapsed)
        );
    }

    #[test]
    #[ignore = "throughput smoke test: cargo test --release -- --ignored --nocapture"]
    fn throughput_publish_worker_pool() {
        const N: usize = 200_000;
        let bus = EventStream::new(Arc::new(Executor::FixedSizedWorkerPool(4)));
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);

        bus.subscribe::<Ping>(move |_: &Ping| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        let start = Instant::now();
        for i in 0..N {
            bus.publish(Ping(i as u32));
        }

        // publish() enqueues into the subscriber's own mailbox and
        // returns immediately on a pooled executor — poll for drain
        // completion instead of assuming publish() blocked.
        let deadline = Instant::now() + Duration::from_secs(30);
        while count.load(Ordering::SeqCst) < N {
            assert!(
                Instant::now() < deadline,
                "events not fully delivered in time"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        let elapsed = start.elapsed();

        println!(
            "[publish/pool]      {N} events in {elapsed:?} = {:.0} events/sec",
            throughput(N, elapsed)
        );
    }
}
