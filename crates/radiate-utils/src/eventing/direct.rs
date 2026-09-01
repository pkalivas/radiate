//! Sibling to `mailbox.rs`/`subscriber.rs`/`hub.rs` — same `Event`/`EventHandler` contract,
//! same `Subscription`/`Schedule` throttling, same pluggable `Executor`, but no mailbox and
//! no per-message boxing. A subscriber is just `Arc<Mutex<H>>`; publishing to it locks the
//! mutex and calls `handle()` directly, instead of enqueueing an envelope onto a channel and
//! having a claim/dispatch dance decide who drains it.
//!
//! Built to A/B against `subscriber.rs`/`hub.rs` (see the throughput tests below), not as a
//! foregone replacement. What dropping the mailbox actually costs, precisely:
//!
//! - **No FIFO ordering guarantee once dispatch runs on more than one pool worker.** The
//!   mailbox's atomic claim means exactly one thread ever drains a given subscriber at a
//!   time, so messages are handled in publish order no matter the pool size. Here, two
//!   events published back-to-back can be submitted to two different workers that race for
//!   the same mutex — whichever wins the lock runs first. Safe under `Executor::Inline`
//!   (dispatch never leaves the publishing thread), not safe under
//!   `Executor::FixedSizedWorkerPool(n>1)` for a subscriber that cares about order.
//! - **No panic isolation, by design.** A handler that panics while holding the lock
//!   poisons it; `.lock().unwrap()` on every later call panics too. Under `Inline` dispatch
//!   that propagates straight up the same call stack that published — "the engine should
//!   stop," per how this variant is meant to be used, not "quietly wall off one subscriber."
//!   Under a worker pool it's weaker than that: the panic unwinds and kills *only* that pool
//!   thread (Rust doesn't crash the process for a panic on a non-main thread by default), so
//!   the publisher's `publish()` call already returned and has no way to know anything
//!   failed. That silent-swallow is inherent to any thread pool without something explicitly
//!   catching and re-surfacing the panic — the old mailbox design's `catch_unwind` existed
//!   as much to compensate for that as for "resilience."

use super::executor::Executor;
use super::subscription::{Subscription, SubscriptionId};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

/// Deliberately *not* `handler.rs`'s `Event`/`EventHandler` — this module's `EventContext` is
/// a different concrete type (wraps this file's `Subscriber`, not `subscriber.rs`'s), and the
/// trait's `handle` signature has to name a concrete context type. Keeping a fully independent
/// pair here means this whole variant stays a self-contained sibling: nothing here can affect
/// the mailbox variant's trait definitions, and this file can be deleted outright with zero
/// impact if the comparison goes the other way.
pub(super) trait Event: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Event for T {}

pub(super) trait EventHandler<E: Event>: Send + 'static {
    fn handle(&mut self, event: &E, ctx: &EventContext<'_, Self>)
    where
        Self: Sized;
}

impl<E, F> EventHandler<E> for F
where
    E: Event,
    F: FnMut(&E) + Send + 'static,
{
    fn handle(&mut self, event: &E, _ctx: &EventContext<'_, Self>) {
        self(event)
    }
}

type Payload = Arc<dyn Any + Send + Sync>;
type Forward = Arc<dyn Fn(Payload) + Send + Sync>;

#[derive(Clone)]
struct Registration {
    forward: Forward,
    subscription: Subscription,
}

type SubscriberList = Arc<Vec<Registration>>;
type SubscriberMap = HashMap<TypeId, SubscriberList>;

/// Borrows the subscriber rather than cloning it — `send` is a plain synchronous call (no
/// `'static` envelope to build, unlike the mailbox variant), so there's nothing to own here.
pub(super) struct EventContext<'a, H>(&'a Subscriber<H>);

impl<H> EventContext<'_, H> {
    pub(super) fn publish<E: Event>(&self, event: E) {
        if let Some(hub) = &self.0.hub {
            hub.publish(event);
        }
    }
}

pub(super) struct Subscriber<H> {
    handler: Arc<Mutex<H>>,
    hub: Option<Hub>,
}

impl<H: Send + 'static> Subscriber<H> {
    pub(super) fn new(handler: H) -> Self {
        Subscriber {
            handler: Arc::new(Mutex::new(handler)),
            hub: None,
        }
    }

    pub(super) fn with_hub(handler: H, hub: Hub) -> Self {
        Subscriber {
            handler: Arc::new(Mutex::new(handler)),
            hub: Some(hub),
        }
    }

    /// Locks the handler and calls it directly — no envelope, no channel, no claim.
    pub(super) fn send<E>(&self, event: &E)
    where
        E: Event,
        H: EventHandler<E>,
    {
        let ctx = EventContext(self);
        self.handler.lock().unwrap().handle(event, &ctx);
    }
}

impl<H> Clone for Subscriber<H> {
    fn clone(&self) -> Self {
        Subscriber {
            handler: Arc::clone(&self.handler),
            hub: self.hub.clone(),
        }
    }
}

#[derive(Clone, Default)]
pub(super) struct Hub {
    subscribers: Arc<RwLock<SubscriberMap>>,
    executor: Arc<Executor>,
}

impl Hub {
    pub(super) fn new() -> Self {
        Hub::default()
    }

    pub(super) fn with_executor(executor: Executor) -> Self {
        Hub {
            subscribers: Arc::default(),
            executor: Arc::new(executor),
        }
    }

    pub(super) fn spawn<H: Send + 'static>(&self, handler: H) -> Subscriber<H> {
        Subscriber::with_hub(handler, self.clone())
    }

    #[inline]
    pub(super) fn publish<E: Event>(&self, event: E) {
        let type_id = TypeId::of::<E>();
        let Some(group) = self.subscribers.read().unwrap().get(&type_id).cloned() else {
            return;
        };

        self.dispatch(&group, Arc::new(event));
    }

    pub(super) fn lazy_publish<E: Event>(&self, f: impl FnOnce() -> E) {
        let type_id = TypeId::of::<E>();
        let Some(group) = self.subscribers.read().unwrap().get(&type_id).cloned() else {
            return;
        };

        let any_due = group
            .iter()
            .any(|registration| registration.subscription.reserve());
        if !any_due {
            return;
        }

        self.dispatch_scheduled(&group, Arc::new(f()));
    }

    pub(super) fn unsubscribe(&self, id: SubscriptionId) {
        let mut subscribers = self.subscribers.write().unwrap();
        for group in subscribers.values_mut() {
            Arc::make_mut(group).retain(|registration| registration.subscription.id() != id);
        }
    }

    pub(super) fn subscribe<E, H>(&self, subscriber: &Subscriber<H>) -> Subscription
    where
        E: Event,
        H: EventHandler<E>,
    {
        let target = subscriber.clone();
        let forward: Forward = Arc::new(move |payload: Payload| {
            if let Ok(event) = payload.downcast::<E>() {
                target.send(event.as_ref());
            }
        });

        self.register::<E>(forward)
    }

    fn register<E: Event>(&self, forward: Forward) -> Subscription {
        let subscription = Subscription::new();
        let registration = Registration {
            forward,
            subscription: subscription.clone(),
        };

        let mut subscribers = self.subscribers.write().unwrap();
        let type_id = TypeId::of::<E>();
        let list = subscribers
            .entry(type_id)
            .or_insert_with(|| Arc::new(Vec::new()));
        let list = Arc::make_mut(list);

        list.retain(|registration| registration.subscription.is_alive());
        list.push(registration);

        subscription
    }

    /// Under `Inline`, call `forward` directly — same shape as `hub.rs`/old `EventStream`'s
    /// dispatch loop, no extra `Arc` clone. A pooled executor needs a `'static` closure to
    /// submit, which means cloning `forward` out of the registration first (and accepting
    /// the ordering caveat in this file's module doc comment).
    fn dispatch(&self, group: &SubscriberList, payload: Payload) {
        for registration in group.iter() {
            if !registration.subscription.is_alive() {
                continue;
            }

            match self.executor.as_ref() {
                Executor::Inline => (registration.forward)(Arc::clone(&payload)),
                Executor::FixedSizedWorkerPool(_) => {
                    let payload = Arc::clone(&payload);
                    let forward = Arc::clone(&registration.forward);
                    self.executor.submit(move || forward(payload));
                }
            }
        }
    }

    fn dispatch_scheduled(&self, group: &SubscriberList, payload: Payload) {
        for registration in group.iter() {
            if !registration.subscription.is_alive() {
                continue;
            }
            if !registration.subscription.take_permit() {
                continue;
            }

            match self.executor.as_ref() {
                Executor::Inline => (registration.forward)(Arc::clone(&payload)),
                Executor::FixedSizedWorkerPool(_) => {
                    let payload = Arc::clone(&payload);
                    let forward = Arc::clone(&registration.forward);
                    self.executor.submit(move || forward(payload));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    struct Add(usize);

    struct Counter {
        total: Arc<AtomicUsize>,
    }
    impl EventHandler<Add> for Counter {
        fn handle(&mut self, event: &Add, _ctx: &EventContext<'_, Self>) {
            self.total.fetch_add(event.0, Ordering::SeqCst);
        }
    }

    #[derive(Clone)]
    struct Ping(u32);
    struct Pong(u32);

    // --- correctness ---

    #[test]
    fn one_handler_one_event_round_trips_through_the_lock() {
        let total = Arc::new(AtomicUsize::new(0));
        let subscriber = Subscriber::new(Counter {
            total: Arc::clone(&total),
        });

        subscriber.send(&Add(3));
        subscriber.send(&Add(4));

        assert_eq!(total.load(Ordering::SeqCst), 7);
    }

    #[test]
    fn a_plain_closure_is_a_handler_via_the_blanket_impl() {
        let total = Arc::new(AtomicUsize::new(0));
        let total_clone = Arc::clone(&total);
        let subscriber = Subscriber::new(move |event: &Add| {
            total_clone.fetch_add(event.0, Ordering::SeqCst);
        });

        subscriber.send(&Add(5));

        assert_eq!(total.load(Ordering::SeqCst), 5);
    }

    #[test]
    fn publish_fans_out_to_every_subscriber() {
        let hub = Hub::new();
        let a_total = Arc::new(AtomicUsize::new(0));
        let b_total = Arc::new(AtomicUsize::new(0));

        let a = hub.spawn({
            let total = Arc::clone(&a_total);
            move |event: &Ping| {
                total.fetch_add(event.0 as usize, Ordering::SeqCst);
            }
        });
        let b = hub.spawn({
            let total = Arc::clone(&b_total);
            move |event: &Ping| {
                total.fetch_add(event.0 as usize, Ordering::SeqCst);
            }
        });

        hub.subscribe::<Ping, _>(&a);
        hub.subscribe::<Ping, _>(&b);

        hub.publish(Ping(3));
        hub.publish(Ping(4));

        assert_eq!(a_total.load(Ordering::SeqCst), 7);
        assert_eq!(b_total.load(Ordering::SeqCst), 7);
    }

    #[test]
    fn a_subscriber_can_react_to_more_than_one_event_type() {
        struct Both {
            pings: Arc<AtomicUsize>,
            pongs: Arc<AtomicUsize>,
        }
        impl EventHandler<Ping> for Both {
            fn handle(&mut self, _: &Ping, _ctx: &EventContext<'_, Self>) {
                self.pings.fetch_add(1, Ordering::SeqCst);
            }
        }
        impl EventHandler<Pong> for Both {
            fn handle(&mut self, _: &Pong, _ctx: &EventContext<'_, Self>) {
                self.pongs.fetch_add(1, Ordering::SeqCst);
            }
        }

        let hub = Hub::new();
        let pings = Arc::new(AtomicUsize::new(0));
        let pongs = Arc::new(AtomicUsize::new(0));
        let subscriber = hub.spawn(Both {
            pings: Arc::clone(&pings),
            pongs: Arc::clone(&pongs),
        });

        hub.subscribe::<Ping, _>(&subscriber);
        hub.subscribe::<Pong, _>(&subscriber);

        hub.publish(Ping(1));
        hub.publish(Pong(2));
        hub.publish(Ping(3));

        assert_eq!(pings.load(Ordering::SeqCst), 2);
        assert_eq!(pongs.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn unsubscribe_stops_further_delivery() {
        let hub = Hub::new();
        let total = Arc::new(AtomicUsize::new(0));
        let subscriber = hub.spawn({
            let total = Arc::clone(&total);
            move |_: &Ping| {
                total.fetch_add(1, Ordering::SeqCst);
            }
        });

        let subscription = hub.subscribe::<Ping, _>(&subscriber);
        hub.publish(Ping(1));
        subscription.unsubscribe();
        hub.publish(Ping(1));

        assert_eq!(total.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn publish_without_a_hub_is_a_silent_noop() {
        struct Emitter;
        impl EventHandler<Add> for Emitter {
            fn handle(&mut self, event: &Add, ctx: &EventContext<'_, Self>) {
                ctx.publish(Add(event.0));
            }
        }

        let subscriber = Subscriber::new(Emitter);
        subscriber.send(&Add(1)); // must not panic despite no hub attached
    }

    #[test]
    fn every_n_schedule_throttles_lazy_publish_delivery() {
        let hub = Hub::new();
        let total = Arc::new(AtomicUsize::new(0));
        let subscriber = hub.spawn({
            let total = Arc::clone(&total);
            move |_: &Ping| {
                total.fetch_add(1, Ordering::SeqCst);
            }
        });

        hub.subscribe::<Ping, _>(&subscriber).schedule(3_usize);

        for i in 0..6 {
            hub.lazy_publish(move || Ping(i));
        }

        assert_eq!(total.load(Ordering::SeqCst), 2);
    }

    // --- the explicit design choice: panics propagate, they aren't isolated ---

    #[test]
    fn a_panicking_handler_poisons_the_lock_instead_of_being_isolated() {
        struct Boom;
        impl EventHandler<Add> for Boom {
            fn handle(&mut self, _: &Add, _ctx: &EventContext<'_, Self>) {
                panic!("simulated failure");
            }
        }

        let subscriber = Subscriber::new(Boom);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            subscriber.send(&Add(1));
        }));
        assert!(result.is_err());

        // Unlike the mailbox variant (which marks itself dead and quietly keeps going),
        // the lock is now poisoned — every subsequent call panics too, cascading rather
        // than isolating.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            subscriber.send(&Add(1));
        }));
        assert!(result.is_err());
    }

    // --- throughput smoke tests: same names/scale as `subscriber.rs`/`hub.rs` for a direct
    // three-way comparison (mailbox vs. direct-lock vs. old `radiate-engines::events`). Run
    // with: cargo test -p radiate-utils --release -- --ignored --nocapture

    fn throughput(count: usize, elapsed: Duration) -> f64 {
        count as f64 / elapsed.as_secs_f64()
    }

    #[test]
    #[ignore = "throughput smoke test: cargo test --release -- --ignored --nocapture"]
    fn throughput_send_single_subscriber() {
        const WARMUP: usize = 100_000;
        const N: usize = 1_000_000;
        let total = Arc::new(AtomicUsize::new(0));
        let subscriber = Subscriber::new(Counter {
            total: Arc::clone(&total),
        });

        for _ in 0..WARMUP {
            subscriber.send(&Add(1));
        }
        let baseline = total.load(Ordering::SeqCst);

        let start = Instant::now();
        for _ in 0..N {
            subscriber.send(&Add(1));
        }
        let elapsed = start.elapsed();

        assert_eq!(total.load(Ordering::SeqCst) - baseline, N);
        println!(
            "[direct/send]            {N} msgs in {elapsed:?} = {:.0} msgs/sec",
            throughput(N, elapsed)
        );
    }

    #[test]
    #[ignore = "throughput smoke test: cargo test --release -- --ignored --nocapture"]
    fn throughput_send_many_subscribers_concurrent_producers() {
        const SUBSCRIBERS: usize = 8;
        const PER_SUBSCRIBER: usize = 200_000;

        let totals: Vec<_> = (0..SUBSCRIBERS)
            .map(|_| Arc::new(AtomicUsize::new(0)))
            .collect();
        let subscribers: Vec<_> = totals
            .iter()
            .map(|total| {
                Subscriber::new(Counter {
                    total: Arc::clone(total),
                })
            })
            .collect();

        let start = Instant::now();
        std::thread::scope(|scope| {
            for subscriber in &subscribers {
                scope.spawn(|| {
                    for _ in 0..PER_SUBSCRIBER {
                        subscriber.send(&Add(1));
                    }
                });
            }
        });
        let elapsed = start.elapsed();

        for total in &totals {
            assert_eq!(total.load(Ordering::SeqCst), PER_SUBSCRIBER);
        }

        let total_msgs = SUBSCRIBERS * PER_SUBSCRIBER;
        println!(
            "[direct/parallel]        {SUBSCRIBERS} subscribers x {PER_SUBSCRIBER} = {total_msgs} msgs in {elapsed:?} = {:.0} msgs/sec",
            throughput(total_msgs, elapsed)
        );
    }

    #[test]
    #[ignore = "throughput smoke test: cargo test --release -- --ignored --nocapture"]
    fn throughput_publish_single_subscriber() {
        const WARMUP: usize = 50_000;
        const N: usize = 500_000;
        let hub = Hub::new();
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);
        let subscriber = hub.spawn(move |_: &Ping| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });
        hub.subscribe::<Ping, _>(&subscriber);

        for i in 0..WARMUP {
            hub.publish(Ping(i as u32));
        }
        let baseline = count.load(Ordering::SeqCst);

        let start = Instant::now();
        for i in 0..N {
            hub.publish(Ping(i as u32));
        }
        let elapsed = start.elapsed();

        assert_eq!(count.load(Ordering::SeqCst) - baseline, N);
        println!(
            "[direct hub/publish 1 sub]  {N} events in {elapsed:?} = {:.0} events/sec",
            throughput(N, elapsed)
        );
    }

    #[test]
    #[ignore = "throughput smoke test: cargo test --release -- --ignored --nocapture"]
    fn throughput_publish_fanout() {
        const WARMUP: usize = 20_000;
        const N: usize = 100_000;
        const SUBSCRIBERS: usize = 8;

        let hub = Hub::new();
        let counts: Vec<_> = (0..SUBSCRIBERS)
            .map(|_| Arc::new(AtomicUsize::new(0)))
            .collect();

        for count in &counts {
            let count = Arc::clone(count);
            let subscriber = hub.spawn(move |_: &Ping| {
                count.fetch_add(1, Ordering::SeqCst);
            });
            hub.subscribe::<Ping, _>(&subscriber);
        }

        for i in 0..WARMUP {
            hub.publish(Ping(i as u32));
        }
        let baseline = counts[0].load(Ordering::SeqCst);

        let start = Instant::now();
        for i in 0..N {
            hub.publish(Ping(i as u32));
        }
        let elapsed = start.elapsed();

        for count in &counts {
            assert_eq!(count.load(Ordering::SeqCst) - baseline, N);
        }

        let deliveries = N * SUBSCRIBERS;
        println!(
            "[direct hub/publish fanout x{SUBSCRIBERS}] {deliveries} deliveries in {elapsed:?} = {:.0} deliveries/sec",
            throughput(deliveries, elapsed)
        );
    }
}
