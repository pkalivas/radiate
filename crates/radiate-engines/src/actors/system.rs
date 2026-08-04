use radiate_core::SmallStr;

use crate::{
    ActorContext, ActorId, Executor,
    actors::{
        MessageHandler,
        actor::{Actor, ActorCell, Addr, Recipient},
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

    /// Spawns `actor` as the singleton for type `A`, registered under its
    /// own type name. A second `spawn::<A>` replaces it in the registry —
    /// same singleton-per-type contract as before, just keyed by name now
    /// instead of a separate `TypeId` map.
    pub fn spawn<A: Actor + 'static>(&self, actor: A) -> Addr<A> {
        self.spawn_named(std::any::type_name::<A>(), actor)
    }

    pub fn spawn_named<A: Actor + 'static>(&self, name: &str, actor: A) -> Addr<A> {
        let actor_ref = self.build_addr(actor, Some(SmallStr::from(name)));
        self.context
            .registry
            .insert(name.to_string(), actor_ref.clone());
        actor_ref
    }

    pub fn listen<A, M>(&self, actor: A)
    where
        A: MessageHandler<M> + 'static,
        M: Send + Clone + 'static,
    {
        let actor_ref = self.spawn(actor);
        self.context.bus.subscribe(actor_ref.recipient::<M>());
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

    fn build_addr<A: Actor + 'static>(&self, actor: A, pid: Option<SmallStr>) -> Addr<A> {
        let (sender, receiver) = std::sync::mpsc::channel();

        Addr {
            sender,
            cell: Arc::new(ActorCell {
                id: ActorId::new(),
                pid,
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

    pub fn publish<M: Send + Clone + 'static>(&self, message: M) {
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

    fn wait_for(signal: &Arc<(Mutex<usize>, Condvar)>, target: usize) {
        let (lock, cv) = &**signal;
        let mut count = lock.lock().unwrap();
        while *count < target {
            let (guard, timeout) = cv.wait_timeout(count, Duration::from_secs(2)).unwrap();
            count = guard;
            if timeout.timed_out() && *count < target {
                panic!("timed out waiting for {target} signals, saw {}", *count);
            }
        }
    }

    // ---------------------------------------------------------------
    // Bare Actor / ActorRef / ActorCell behavior
    // ---------------------------------------------------------------

    struct Counter {
        seen: Arc<Mutex<Vec<i32>>>,
        signal: Arc<(Mutex<usize>, Condvar)>,
    }

    impl Actor for Counter {}

    impl MessageHandler<i32> for Counter {
        fn handle(&mut self, message: i32, _ctx: &ActorContext) {
            self.seen.lock().unwrap().push(message);
            let (count, cv) = &*self.signal;
            *count.lock().unwrap() += 1;
            cv.notify_all();
        }
    }

    #[test]
    fn tell_delivers_a_single_message() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let actor_ref = system.spawn(Counter {
            seen: Arc::clone(&seen),
            signal: Arc::clone(&signal),
        });

        actor_ref.send(7);
        wait_for(&signal, 1);

        assert_eq!(*seen.lock().unwrap(), vec![7]);
    }

    #[test]
    fn messages_are_delivered_in_fifo_order() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let actor_ref = system.spawn(Counter {
            seen: Arc::clone(&seen),
            signal: Arc::clone(&signal),
        });

        const N: usize = 200;
        for i in 0..N as i32 {
            actor_ref.send(i);
        }
        wait_for(&signal, N);

        let expected: Vec<i32> = (0..N as i32).collect();
        assert_eq!(*seen.lock().unwrap(), expected);
    }

    #[test]
    fn cloned_refs_all_feed_the_same_mailbox() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let actor_ref = system.spawn(Counter {
            seen: Arc::clone(&seen),
            signal: Arc::clone(&signal),
        });

        let a = actor_ref.clone();
        let b = actor_ref.clone();
        let ha = std::thread::spawn(move || a.send(1));
        let hb = std::thread::spawn(move || b.send(2));
        ha.join().unwrap();
        hb.join().unwrap();

        wait_for(&signal, 2);

        let mut got = seen.lock().unwrap().clone();
        got.sort();
        assert_eq!(got, vec![1, 2]);
    }

    // ---------------------------------------------------------------
    // Panic isolation: a panicking message doesn't poison the actor
    // or stop subsequent messages from being processed.
    // ---------------------------------------------------------------

    struct Flaky {
        seen: Arc<Mutex<Vec<i32>>>,
        signal: Arc<(Mutex<usize>, Condvar)>,
    }

    impl Actor for Flaky {}

    impl MessageHandler<i32> for Flaky {
        fn handle(&mut self, message: i32, _ctx: &ActorContext) {
            if message == 2 {
                panic!("boom");
            }
            self.seen.lock().unwrap().push(message);
            let (count, cv) = &*self.signal;
            *count.lock().unwrap() += 1;
            cv.notify_all();
        }
    }

    #[test]
    fn actor_survives_a_panicking_message_and_keeps_processing() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let actor_ref = system.spawn(Flaky {
            seen: Arc::clone(&seen),
            signal: Arc::clone(&signal),
        });

        actor_ref.send(1);
        actor_ref.send(2); // panics, should not take the actor down
        actor_ref.send(3);

        wait_for(&signal, 2); // only 1 and 3 signal

        assert_eq!(*seen.lock().unwrap(), vec![1, 3]);
    }

    // ---------------------------------------------------------------
    // stop(): on_stop runs once, after everything queued ahead of it;
    // sends after that dead-letter instead of being delivered.
    // ---------------------------------------------------------------

    struct Stoppable {
        seen: Arc<Mutex<Vec<i32>>>,
        signal: Arc<(Mutex<usize>, Condvar)>,
        stop_count: Arc<(Mutex<usize>, Condvar)>,
    }

    impl Actor for Stoppable {
        fn on_stop(&mut self, _ctx: &ActorContext) {
            let (count, cv) = &*self.stop_count;
            *count.lock().unwrap() += 1;
            cv.notify_all();
        }
    }

    impl MessageHandler<i32> for Stoppable {
        fn handle(&mut self, message: i32, _ctx: &ActorContext) {
            self.seen.lock().unwrap().push(message);
            let (count, cv) = &*self.signal;
            *count.lock().unwrap() += 1;
            cv.notify_all();
        }
    }

    #[test]
    fn stop_runs_once_after_queued_messages_then_dead_letters_further_sends() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));
        let stop_count = Arc::new((Mutex::new(0), Condvar::new()));

        let dead_letters = Arc::new(Mutex::new(Vec::new()));
        let dl_signal = Arc::new((Mutex::new(0), Condvar::new()));
        let dl2 = Arc::clone(&dead_letters);
        let dls2 = Arc::clone(&dl_signal);
        system.subscribe::<DeadLetter>(move |dl: &DeadLetter, _ctx: &ActorContext| {
            dl2.lock().unwrap().push(dl.message_type);
            let (count, cv) = &*dls2;
            *count.lock().unwrap() += 1;
            cv.notify_all();
        });

        let actor_ref = system.spawn(Stoppable {
            seen: Arc::clone(&seen),
            signal: Arc::clone(&signal),
            stop_count: Arc::clone(&stop_count),
        });

        actor_ref.send(1);
        actor_ref.send(2);
        wait_for(&signal, 2);

        actor_ref.stop();
        wait_for(&stop_count, 1);

        // Idempotent: a second stop() must not run on_stop again.
        actor_ref.stop();
        assert_eq!(*stop_count.0.lock().unwrap(), 1);

        // Sends after stop() are dead-lettered, not delivered.
        actor_ref.send(3);
        wait_for(&dl_signal, 1);

        assert_eq!(*seen.lock().unwrap(), vec![1, 2]);
        assert_eq!(
            *dead_letters.lock().unwrap(),
            vec![std::any::type_name::<i32>()]
        );
    }

    // ---------------------------------------------------------------
    // DomainBus: subscribe/publish fan-out
    // ---------------------------------------------------------------

    #[derive(Clone, Debug, PartialEq)]
    struct Counted(i32);

    #[derive(Clone, Debug, PartialEq)]
    struct Warning(&'static str);

    #[test]
    fn subscribe_and_publish_delivers_message() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let seen2 = Arc::clone(&seen);
        let signal2 = Arc::clone(&signal);
        system.subscribe::<Counted>(move |msg, _ctx| {
            seen2.lock().unwrap().push(msg.0);
            let (count, cv) = &*signal2;
            *count.lock().unwrap() += 1;
            cv.notify_all();
        });

        system.context().publish(Counted(42));
        wait_for(&signal, 1);

        assert_eq!(*seen.lock().unwrap(), vec![42]);
    }

    #[test]
    fn unrelated_message_types_do_not_cross_wires() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let seen2 = Arc::clone(&seen);
        let signal2 = Arc::clone(&signal);
        system.subscribe::<Counted>(move |msg, _ctx| {
            seen2.lock().unwrap().push(msg.0);
            let (count, cv) = &*signal2;
            *count.lock().unwrap() += 1;
            cv.notify_all();
        });

        // Nobody subscribed to Warning — should be a silent no-op.
        system.context().publish(Warning("disk almost full"));
        system.context().publish(Counted(1));
        wait_for(&signal, 1);

        assert_eq!(*seen.lock().unwrap(), vec![1]);
    }

    #[test]
    fn multiple_subscribers_of_same_type_all_receive() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let seen_a = Arc::new(Mutex::new(Vec::new()));
        let seen_b = Arc::new(Mutex::new(Vec::new()));
        let signal_a = Arc::new((Mutex::new(0), Condvar::new()));
        let signal_b = Arc::new((Mutex::new(0), Condvar::new()));

        let sa = Arc::clone(&seen_a);
        let siga = Arc::clone(&signal_a);
        system.subscribe::<Counted>(move |msg, _ctx| {
            sa.lock().unwrap().push(msg.0);
            let (count, cv) = &*siga;
            *count.lock().unwrap() += 1;
            cv.notify_all();
        });

        let sb = Arc::clone(&seen_b);
        let sigb = Arc::clone(&signal_b);
        system.subscribe::<Counted>(move |msg, _ctx| {
            sb.lock().unwrap().push(msg.0);
            let (count, cv) = &*sigb;
            *count.lock().unwrap() += 1;
            cv.notify_all();
        });

        system.context().publish(Counted(9));
        wait_for(&signal_a, 1);
        wait_for(&signal_b, 1);

        assert_eq!(*seen_a.lock().unwrap(), vec![9]);
        assert_eq!(*seen_b.lock().unwrap(), vec![9]);
    }

    #[test]
    fn publish_with_no_subscribers_does_not_panic() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        system.context().publish(Counted(1)); // should just be dropped
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
    // AnyActorRef: erasure shouldn't break supervision no-ops
    // ---------------------------------------------------------------

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
            system.context().publish(Counted(i as i32));
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

    #[test]
    fn tell_to_a_dropped_actor_publishes_dead_letter() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let seen2 = Arc::clone(&seen);
        let signal2 = Arc::clone(&signal);
        system.subscribe::<DeadLetter>(move |dl: &DeadLetter, _ctx: &ActorContext| {
            seen2.lock().unwrap().push(dl.message_type);
            let (count, cv) = &*signal2;
            *count.lock().unwrap() += 1;
            cv.notify_all();
        });

        let actor_ref = {
            // actor_ref's own scope ends here; nothing else holds a clone, so
            // its ActorCell (and Receiver) drop once this block exits.
            let temp = system.spawn(Counter {
                seen: Arc::new(Mutex::new(Vec::new())),
                signal: Arc::new((Mutex::new(0), Condvar::new())),
            });
            temp.clone() // clone the ref itself so `sender` outlives the cell — see note
        };

        actor_ref.send(1);
        wait_for(&signal, 1);

        assert_eq!(*seen.lock().unwrap(), vec![std::any::type_name::<i32>()]);
    }

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
