use radiate_core::SmallStr;

use crate::{
    ActorContext, ActorId, Executor,
    actors::{
        MessageHandler, ProcessId,
        actor::{Actor, ActorCell, Addr, Recipient, WeakAddr},
        context::ActorRegistry,
        handler::FnActor,
    },
};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::{Arc, RwLock, atomic::AtomicBool};

#[derive(Clone)]
pub struct ActorSystem {
    context: ActorContext,
}

impl ActorSystem {
    pub fn new(executor: Arc<Executor>) -> Self {
        ActorSystem {
            context: ActorContext {
                executor,
                bus: Arc::new(MessageBus::default()),
                registry: Arc::new(ActorRegistry::default()),
                parent: None,
            },
        }
    }

    pub fn context(&self) -> ActorContext {
        self.context.clone()
    }

    pub fn spawn_fn<A, F>(&self, pid: impl Into<ProcessId>, f: F) -> Addr<A>
    where
        A: Actor + 'static,
        F: FnOnce() -> A,
    {
        let actor = f();
        let addr = self
            .context
            .create(Some(pid.into()), |_: &WeakAddr<A>| actor);

        addr
    }

    /// Spawns `actor` as the singleton for type `A`, registered under its
    /// own type name. A second `spawn::<A>` replaces it in the registry —
    /// same singleton-per-type contract as before, just keyed by name now
    /// instead of a separate `TypeId` map.
    pub fn spawn<A: Actor + 'static>(&self, actor: A) -> Addr<A> {
        self.spawn_named(std::any::type_name::<A>(), actor)
    }

    pub fn spawn_named<A: Actor + 'static>(&self, name: impl Into<ProcessId>, actor: A) -> Addr<A> {
        let pid = name.into();
        let actor_ref = self.build_addr(actor, Some(pid.clone()));
        self.context.registry.insert(pid, actor_ref.clone());
        actor_ref
    }

    pub fn create<A, F>(&self, f: F) -> Addr<A>
    where
        A: Actor + 'static,
        F: FnOnce(&WeakAddr<A>) -> A,
    {
        self.context.create(None, f)
    }

    pub fn subscribe<M: Send + Clone + 'static>(
        &self,
        mut handler: impl FnMut(&M, &ActorContext) + Send + Sync + 'static,
    ) {
        self.subscribe_with::<M, _>(move |message, ctx| handler(&message, ctx));
    }

    fn subscribe_with<M, F>(&self, f: F)
    where
        M: Send + Clone + 'static,
        F: FnMut(M, &ActorContext) + Send + Sync + 'static,
    {
        let actor = FnActor {
            handler: Box::new(f),
        };
        let actor_ref = self.build_addr(actor, None);
        self.context.bus.subscribe(actor_ref.recipient::<M>());
    }

    fn build_addr<A: Actor + 'static>(&self, actor: A, pid: Option<ProcessId>) -> Addr<A> {
        let (sender, receiver) = std::sync::mpsc::channel();

        Addr {
            sender,
            cell: Arc::new(ActorCell {
                id: ActorId::new(),
                pid: pid.map(|p| p.clone()),
                actor: Arc::new(Mutex::new(actor)),
                receiver,
                scheduled: AtomicBool::new(false),
                stopped: AtomicBool::new(false),
                context: ActorContext {
                    bus: Arc::clone(&self.context.bus),
                    executor: Arc::clone(&self.context.executor),
                    registry: Arc::clone(&self.context.registry),
                    parent: None,
                },
            }),
        }
    }
}

impl From<(Arc<Executor>, Arc<MessageBus>)> for ActorSystem {
    fn from((executor, bus): (Arc<Executor>, Arc<MessageBus>)) -> Self {
        ActorSystem {
            context: ActorContext {
                executor,
                bus,
                registry: Arc::new(ActorRegistry::default()),
                parent: None,
            },
        }
    }
}

impl Default for ActorSystem {
    fn default() -> Self {
        ActorSystem::new(Arc::new(Executor::default()))
    }
}

#[derive(Default)]
pub struct MessageBus {
    subscribers: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

impl MessageBus {
    pub fn subscribe<M: Send + 'static>(&self, recipient: Recipient<M>) {
        let mut registry = self.subscribers.write().unwrap();

        registry
            .entry(TypeId::of::<M>())
            .or_insert_with(|| Box::new(Vec::<Recipient<M>>::new()) as Box<dyn Any + Send + Sync>)
            .downcast_mut::<Vec<Recipient<M>>>()
            .expect("TypeId key always matches Vec<Recipient<M>> by construction")
            .push(recipient);
    }

    pub fn send<M: Send + Clone + 'static>(&self, message: M) {
        let registry = self.subscribers.read().unwrap();
        if let Some(group) = registry
            .get(&TypeId::of::<M>())
            .and_then(|b| b.downcast_ref::<Vec<Recipient<M>>>())
        {
            for recipient in group {
                recipient.tell(message.clone());
            }
        }
    }

    pub fn has_subscribers<M: Send + 'static>(&self) -> bool {
        self.subscribers
            .read()
            .unwrap()
            .contains_key(&TypeId::of::<M>())
    }
}

#[cfg(test)]
mod tests {
    use crate::actors::message::DeadLetter;

    use super::*;
    use std::{
        sync::atomic::{AtomicU64, Ordering},
        time::Duration,
    };
    use std::{
        sync::{Condvar, Mutex},
        time::Instant,
    };

    // ---------------------------------------------------------------
    // Shared test helpers.
    //
    // `Signal` + `Recorder<T>` replace the "lock a Mutex<usize>, increment,
    // notify_all" dance that used to be hand-rolled inside every actor's
    // `handle()` below — tests should read as "record this, then wait for
    // N," not re-derive a condvar every time.
    //
    // `wait_until` is a different tool for a different need: an arbitrary
    // predicate over something that isn't a `Signal` (e.g. a raw
    // `AtomicU64` total in the throughput benchmarks) — reach for `Signal`
    // when waiting on "N of this specific thing happened," and
    // `wait_until` when the condition doesn't fit that shape.
    // ---------------------------------------------------------------

    const DEFAULT_TIMEOUT: Duration = Duration::from_secs(2);

    #[derive(Default)]
    struct Signal {
        count: Mutex<usize>,
        cv: Condvar,
    }

    impl Signal {
        fn bump(&self) {
            *self.count.lock().unwrap() += 1;
            self.cv.notify_all();
        }

        fn wait_for(&self, target: usize) {
            let mut count = self.count.lock().unwrap();
            while *count < target {
                let (guard, timeout) = self.cv.wait_timeout(count, DEFAULT_TIMEOUT).unwrap();
                count = guard;
                if timeout.timed_out() && *count < target {
                    panic!("timed out waiting for {target} signals, saw {}", *count);
                }
            }
        }
    }

    /// Records values pushed to it and lets a test block until `target`
    /// have arrived. Defaults to `T = i32` since that covers most actor
    /// payloads below; the dead-letter tests use `Recorder<&'static str>`.
    struct Recorder<T = i32> {
        seen: Mutex<Vec<T>>,
        signal: Signal,
    }

    impl<T> Default for Recorder<T> {
        fn default() -> Self {
            Recorder {
                seen: Mutex::new(Vec::new()),
                signal: Signal::default(),
            }
        }
    }

    impl<T: Clone> Recorder<T> {
        fn record(&self, value: T) {
            self.seen.lock().unwrap().push(value);
            self.signal.bump();
        }

        fn wait_for(&self, target: usize) {
            self.signal.wait_for(target);
        }

        fn seen(&self) -> Vec<T> {
            self.seen.lock().unwrap().clone()
        }
    }

    fn wait_until<F: Fn() -> bool>(timeout: Duration, cond: F) -> bool {
        let start = Instant::now();
        while !cond() {
            if start.elapsed() > timeout {
                return false;
            }
            std::thread::yield_now();
        }
        true
    }

    // ---------------------------------------------------------------
    // Bare Actor / Addr / ActorCell behavior
    // ---------------------------------------------------------------

    struct Counter {
        recorder: Arc<Recorder>,
    }

    impl Actor for Counter {}

    impl MessageHandler<i32> for Counter {
        fn handle(&mut self, message: i32, _ctx: &ActorContext) {
            self.recorder.record(message);
        }
    }

    #[test]
    fn tell_delivers_a_single_message() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let recorder = Arc::new(Recorder::default());

        let actor_ref = system.spawn(Counter {
            recorder: Arc::clone(&recorder),
        });

        actor_ref.send(7);
        recorder.wait_for(1);

        assert_eq!(recorder.seen(), vec![7]);
    }

    #[test]
    fn messages_are_delivered_in_fifo_order() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let recorder = Arc::new(Recorder::default());

        let actor_ref = system.spawn(Counter {
            recorder: Arc::clone(&recorder),
        });

        const N: usize = 200;
        for i in 0..N as i32 {
            actor_ref.send(i);
        }
        recorder.wait_for(N);

        let expected: Vec<i32> = (0..N as i32).collect();
        assert_eq!(recorder.seen(), expected);
    }

    #[test]
    fn cloned_refs_all_feed_the_same_mailbox() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let recorder = Arc::new(Recorder::default());

        let actor_ref = system.spawn(Counter {
            recorder: Arc::clone(&recorder),
        });

        let a = actor_ref.clone();
        let b = actor_ref.clone();
        let ha = std::thread::spawn(move || a.send(1));
        let hb = std::thread::spawn(move || b.send(2));
        ha.join().unwrap();
        hb.join().unwrap();

        recorder.wait_for(2);

        let mut got = recorder.seen();
        got.sort();
        assert_eq!(got, vec![1, 2]);
    }

    // ---------------------------------------------------------------
    // Panic isolation: a panicking message doesn't poison the actor
    // or stop subsequent messages from being processed.
    // ---------------------------------------------------------------

    struct Flaky {
        recorder: Arc<Recorder>,
    }

    impl Actor for Flaky {}

    impl MessageHandler<i32> for Flaky {
        fn handle(&mut self, message: i32, _ctx: &ActorContext) {
            if message == 2 {
                panic!("boom");
            }
            self.recorder.record(message);
        }
    }

    #[test]
    fn actor_survives_a_panicking_message_and_keeps_processing() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let recorder = Arc::new(Recorder::default());

        let actor_ref = system.spawn(Flaky {
            recorder: Arc::clone(&recorder),
        });

        actor_ref.send(1);
        actor_ref.send(2); // panics, should not take the actor down
        actor_ref.send(3);

        recorder.wait_for(2); // only 1 and 3 record

        assert_eq!(recorder.seen(), vec![1, 3]);
    }

    // ---------------------------------------------------------------
    // stop(): on_stop runs once, after everything queued ahead of it;
    // sends after that dead-letter instead of being delivered.
    // ---------------------------------------------------------------

    struct Stoppable {
        recorder: Arc<Recorder>,
        stop_signal: Arc<Signal>,
    }

    impl Actor for Stoppable {
        fn on_stop(&mut self, _ctx: &ActorContext) {
            self.stop_signal.bump();
        }
    }

    impl MessageHandler<i32> for Stoppable {
        fn handle(&mut self, message: i32, _ctx: &ActorContext) {
            self.recorder.record(message);
        }
    }

    #[test]
    fn stop_runs_once_after_queued_messages_then_dead_letters_further_sends() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let recorder = Arc::new(Recorder::default());
        let stop_signal = Arc::new(Signal::default());
        let dead_letters = Arc::new(Recorder::<&'static str>::default());

        let dl = Arc::clone(&dead_letters);
        system.subscribe::<DeadLetter>(move |msg: &DeadLetter, _ctx: &ActorContext| {
            dl.record(msg.message_type);
        });

        let actor_ref = system.spawn(Stoppable {
            recorder: Arc::clone(&recorder),
            stop_signal: Arc::clone(&stop_signal),
        });

        actor_ref.send(1);
        actor_ref.send(2);
        recorder.wait_for(2);

        actor_ref.stop();
        stop_signal.wait_for(1);

        // Idempotent: a second stop() must not run on_stop again.
        actor_ref.stop();
        assert_eq!(*stop_signal.count.lock().unwrap(), 1);

        // Sends after stop() are dead-lettered, not delivered.
        actor_ref.send(3);
        dead_letters.wait_for(1);

        assert_eq!(recorder.seen(), vec![1, 2]);
        assert_eq!(dead_letters.seen(), vec![std::any::type_name::<i32>()]);
    }

    // ---------------------------------------------------------------
    // MessageBus: subscribe/publish fan-out
    // ---------------------------------------------------------------

    #[derive(Clone, Debug, PartialEq)]
    struct Counted(i32);

    #[derive(Clone, Debug, PartialEq)]
    struct Warning(&'static str);

    #[test]
    fn subscribe_and_publish_delivers_message() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let recorder = Arc::new(Recorder::default());

        let r = Arc::clone(&recorder);
        system.subscribe::<Counted>(move |msg, _ctx| r.record(msg.0));

        system.context().send(Counted(42));
        recorder.wait_for(1);

        assert_eq!(recorder.seen(), vec![42]);
    }

    #[test]
    fn unrelated_message_types_do_not_cross_wires() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let recorder = Arc::new(Recorder::default());

        let r = Arc::clone(&recorder);
        system.subscribe::<Counted>(move |msg, _ctx| r.record(msg.0));

        // Nobody subscribed to Warning — should be a silent no-op.
        system.context().send(Warning("disk almost full"));
        system.context().send(Counted(1));
        recorder.wait_for(1);

        assert_eq!(recorder.seen(), vec![1]);
    }

    #[test]
    fn multiple_subscribers_of_same_type_all_receive() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let recorder_a = Arc::new(Recorder::default());
        let recorder_b = Arc::new(Recorder::default());

        let ra = Arc::clone(&recorder_a);
        system.subscribe::<Counted>(move |msg, _ctx| ra.record(msg.0));

        let rb = Arc::clone(&recorder_b);
        system.subscribe::<Counted>(move |msg, _ctx| rb.record(msg.0));

        system.context().send(Counted(9));
        recorder_a.wait_for(1);
        recorder_b.wait_for(1);

        assert_eq!(recorder_a.seen(), vec![9]);
        assert_eq!(recorder_b.seen(), vec![9]);
    }

    #[test]
    fn publish_with_no_subscribers_does_not_panic() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        system.context().send(Counted(1)); // should just be dropped
    }

    #[test]
    fn has_subscribers_reflects_registration_state() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        assert!(!system.context().has_subscribers::<Counted>());

        system.subscribe::<Counted>(|_msg, _ctx| {});

        assert!(system.context().has_subscribers::<Counted>());
        assert!(!system.context().has_subscribers::<Warning>());
    }

    // ---------------------------------------------------------------
    // Dead letters are reachable only via the `stopped` flag (see
    // `stop_runs_once_...` above), not via a genuinely-dropped `Receiver`.
    //
    // There used to be a test here trying to construct "send to an actor
    // whose ActorCell has already been dropped elsewhere." It's not
    // fixable, because it's not reachable: `Addr::send` needs `self.cell`
    // to run at all (checks `stopped`, calls `try_claim()`), so any `Addr`
    // capable of calling `.send()` inherently holds a strong reference to
    // the very `ActorCell`/`Receiver` pair it would need to already be
    // gone. There's no sequence of drops or clones that produces a
    // callable `Addr` with an already-dropped cell — removed rather than
    // left red chasing a scenario the type system doesn't allow.
    // ---------------------------------------------------------------

    // ---------------------------------------------------------------
    // Registry-removal identity: stop() on an actor whose registry slot
    // was already overwritten by a newer spawn::<A>() of the same type
    // must not evict the newer, still-live occupant.
    // ---------------------------------------------------------------

    struct Named;
    impl Actor for Named {}

    #[test]
    fn stop_on_an_evicted_registry_slot_does_not_delete_the_current_occupant() {
        let system = ActorSystem::new(Arc::new(Executor::default()));

        let first = system.spawn(Named);
        let second = system.spawn(Named); // same type -> same registry key, evicts `first`

        first.stop(); // `first` was already evicted from the registry -- must be a no-op there

        let current = system.context().actor::<Named>();
        assert!(
            current.is_some(),
            "second actor's registry entry was wrongly deleted by an unrelated stop()"
        );
        assert_eq!(current.unwrap().id(), second.id());
    }

    // ---------------------------------------------------------------
    // WeakAddr: safe self-reference via ActorSystem::create /
    // ActorContext::create (Arc::new_cyclic under the hood).
    // ---------------------------------------------------------------

    struct SelfPinger {
        weak_self: WeakAddr<SelfPinger>,
        recorder: Arc<Recorder>,
    }

    impl Actor for SelfPinger {}

    impl MessageHandler<i32> for SelfPinger {
        fn handle(&mut self, message: i32, _ctx: &ActorContext) {
            self.recorder.record(message);

            if message < 3
                && let Some(me) = self.weak_self.upgrade()
            {
                me.send(message + 1);
            }
        }
    }

    #[test]
    fn actor_can_store_and_use_its_own_weak_address() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let recorder = Arc::new(Recorder::default());

        let r = Arc::clone(&recorder);
        let addr = system.create(move |weak_self: &WeakAddr<SelfPinger>| SelfPinger {
            weak_self: weak_self.clone(),
            recorder: r,
        });

        addr.send(0);
        recorder.wait_for(4); // 0, 1, 2, 3 -- each handle() re-sends to itself via the weak addr

        assert_eq!(recorder.seen(), vec![0, 1, 2, 3]);
    }

    struct Empty;
    impl Actor for Empty {}

    #[test]
    fn weak_addr_upgrade_returns_none_once_every_strong_addr_is_dropped() {
        let system = ActorSystem::new(Arc::new(Executor::default()));

        let weak = {
            // `create` doesn't touch the registry, so once `addr` drops here
            // with nothing else holding a clone, the cell is genuinely gone.
            let addr = system.create(|_weak_self: &WeakAddr<Empty>| Empty);
            addr.downgrade()
        };

        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn weak_addr_upgrade_returns_none_after_stop_even_if_a_strong_clone_survives() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let addr = system.create(|_weak_self: &WeakAddr<Empty>| Empty);
        let weak = addr.downgrade();

        addr.stop(); // `addr` itself is still alive -- strong_count stays > 0

        assert!(
            weak.upgrade().is_none(),
            "a stopped actor shouldn't upgrade just because some other clone kept it alive"
        );
    }

    // ---------------------------------------------------------------
    // Throughput benchmarks, not correctness tests: they print a number
    // and assert loosely (did we hit the target within a generous
    // timeout) rather than an exact value. Both request a
    // `FixedSizedWorkerPool` of a specific size, but `get_thread_pool`
    // (radiate-core) is a single global `OnceLock`, not keyed by size --
    // whichever of these two tests runs first fixes the pool size for the
    // rest of the test binary's process. Numbers are indicative, not
    // exactly reproducible run-to-run if execution order changes.
    // ---------------------------------------------------------------

    #[test]
    fn fan_out_to_many_subscribers_throughput() {
        const N: u64 = 20_000;
        const SUBSCRIBERS: usize = 50;

        let system = ActorSystem::new(Arc::new(Executor::FixedSizedWorkerPool(4)));
        let total_received = Arc::new(AtomicU64::new(0));

        for _ in 0..SUBSCRIBERS {
            let total = Arc::clone(&total_received);
            system.subscribe::<Counted>(move |_msg: &Counted, _ctx: &ActorContext| {
                total.fetch_add(1, Ordering::Relaxed);
            });
        }

        let target = N * SUBSCRIBERS as u64;

        let start = Instant::now();
        for i in 0..N {
            system.context().send(Counted(i as i32));
        }
        let ok = wait_until(Duration::from_secs(20), || {
            total_received.load(Ordering::Relaxed) == target
        });
        let elapsed = start.elapsed();

        assert!(
            ok,
            "timed out: expected {target} total deliveries, saw {}",
            total_received.load(Ordering::Relaxed)
        );

        let throughput = target as f64 / elapsed.as_secs_f64();
        println!(
            "[fan-out]    {N} messages x {SUBSCRIBERS} subscribers = {target} deliveries in {elapsed:?} ({throughput:.0} deliveries/sec)"
        );
    }

    // ---------------------------------------------------------------
    // Cross-"engine" random-routing throughput, mirroring a common actor
    // framework benchmark shape (e.g. hollywood's engine benchmark): N
    // independent ActorSystems ("engines"), each with many actors;
    // concurrent sender threads pick a random engine, a random *other*
    // engine as target, and a random actor within it, and hammer it with
    // messages for a fixed duration. Verifies no message loss and reports
    // throughput.
    // ---------------------------------------------------------------

    #[derive(Clone)]
    struct BenchMessage;

    struct BenchActor {
        received: Arc<AtomicU64>,
    }

    impl Actor for BenchActor {}

    impl MessageHandler<BenchMessage> for BenchActor {
        fn handle(&mut self, _message: BenchMessage, _ctx: &ActorContext) {
            self.received.fetch_add(1, Ordering::Relaxed);
        }
    }

    struct BenchEngine {
        actors: Vec<Addr<BenchActor>>,
    }

    // Dependency-free xorshift64 so each sender thread gets its own cheap,
    // deterministic-per-seed index stream without pulling in `rand`.
    struct Xorshift64(u64);

    impl Xorshift64 {
        fn next_bounded(&mut self, bound: usize) -> usize {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            (self.0 as usize) % bound
        }
    }

    #[test]
    fn cross_engine_random_routing_throughput() {
        const ENGINES: usize = 10;
        const ACTORS_PER_ENGINE: usize = 2000;
        const SENDERS: usize = 20;
        const DURATION: Duration = Duration::from_secs(10);

        let received = Arc::new(AtomicU64::new(0));
        let dead_letters = Arc::new(AtomicU64::new(0));

        let worker_pool = Arc::new(Executor::FixedSizedWorkerPool(
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4),
        ));

        let engines: Vec<BenchEngine> = (0..ENGINES)
            .map(|_| {
                let system = ActorSystem::new(Arc::clone(&worker_pool));

                let dl = Arc::clone(&dead_letters);
                system.subscribe::<DeadLetter>(move |_msg: &DeadLetter, _ctx: &ActorContext| {
                    dl.fetch_add(1, Ordering::Relaxed);
                });

                let actors = (0..ACTORS_PER_ENGINE)
                    .map(|_| {
                        system.spawn(BenchActor {
                            received: Arc::clone(&received),
                        })
                    })
                    .collect();

                BenchEngine { actors }
            })
            .collect();

        let engines = Arc::new(engines);
        let sent = Arc::new(AtomicU64::new(0));
        let start = Instant::now();
        let deadline = start + DURATION;

        let handles: Vec<_> = (0..SENDERS)
            .map(|seed| {
                let engines = Arc::clone(&engines);
                let sent = Arc::clone(&sent);
                std::thread::spawn(move || {
                    let mut rng = Xorshift64(seed as u64 * 2 + 1);
                    while Instant::now() < deadline {
                        let from = rng.next_bounded(engines.len());
                        let mut to = rng.next_bounded(engines.len());
                        if engines.len() > 1 {
                            while to == from {
                                to = rng.next_bounded(engines.len());
                            }
                        }

                        let target_engine = &engines[to];
                        let actor_idx = rng.next_bounded(target_engine.actors.len());
                        target_engine.actors[actor_idx].send(BenchMessage);
                        sent.fetch_add(1, Ordering::Relaxed);
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        let sent_total = sent.load(Ordering::Relaxed);
        let drained = wait_until(Duration::from_secs(10), || {
            received.load(Ordering::Relaxed) == sent_total
        });
        let elapsed = start.elapsed();
        let received_total = received.load(Ordering::Relaxed);

        println!(
            "[cross-engine] {ENGINES} engines x {ACTORS_PER_ENGINE} actors, {SENDERS} senders, {elapsed:?}: sent={sent_total} received={received_total} ({:.0} msgs/sec) dead_letters={}",
            received_total as f64 / elapsed.as_secs_f64(),
            dead_letters.load(Ordering::Relaxed)
        );

        assert!(
            drained,
            "timed out waiting for all sent messages to be received"
        );
        assert_eq!(sent_total, received_total, "message loss detected");
    }
}
