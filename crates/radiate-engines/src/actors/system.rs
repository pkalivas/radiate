use crate::{
    ActorContext, ActorId, Executor,
    actors::{
        MessageHandler,
        actor::{Actor, ActorCell, ActorRef, Recipient},
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

    /// Spawns `actor` and subscribes it to the bus for `M` — the shared
    /// instance forwards every published `M` into its own mailbox via
    /// `MessageHandler<M>::handle`.
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

    /// Spawns `actor` as the singleton for type `A`, registered under its
    /// own type name. A second `spawn::<A>` replaces it in the registry —
    /// same singleton-per-type contract as before, just keyed by name now
    /// instead of a separate `TypeId` map.
    pub fn spawn<A: Actor + 'static>(&self, actor: A) -> ActorRef<A> {
        self.spawn_named(std::any::type_name::<A>(), actor)
    }

    pub fn spawn_named<A: Actor + 'static>(&self, name: &str, actor: A) -> ActorRef<A> {
        let actor_ref = self.build_ref(actor);
        self.context
            .registry
            .insert(name.to_string(), actor_ref.clone());
        actor_ref
    }

    pub fn publish<M: Send + Clone + 'static>(&self, message: M) {
        self.context.bus.publish(message);
    }

    pub fn has_subscribers<M: Send + 'static>(&self) -> bool {
        self.context.bus.has_subscribers::<M>()
    }

    pub fn lazy_publish<M: Send + Clone + 'static>(&self, func: impl FnOnce() -> M) {
        if self.has_subscribers::<M>() {
            self.publish(func());
        }
    }

    /// Spawns an unregistered `FnActor<M>` wired to call `f` on each
    /// delivery, and subscribes its `Recipient<M>` to the bus. The shared
    /// plumbing behind both `subscribe` (closure reacts inline) and
    /// `listen` (closure forwards to another actor).
    fn subscribe_with<M, F>(&self, f: F)
    where
        M: Send + Clone + 'static,
        F: FnMut(M, &ActorContext) + Send + Sync + 'static,
    {
        let actor = FnActor {
            handler: Box::new(f),
        };
        let actor_ref = self.build_ref(actor);
        self.context.bus.subscribe(actor_ref.recipient::<M>());
    }

    fn build_ref<A: Actor + 'static>(&self, actor: A) -> ActorRef<A> {
        let (sender, receiver) = std::sync::mpsc::channel();

        ActorRef {
            sender,
            cell: Arc::new(ActorCell {
                id: ActorId::new(),
                actor: Arc::new(Mutex::new(actor)),
                receiver: Arc::new(Mutex::new(receiver)),
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

// impl Deref for ActorSystem {
//     type Target = ActorContext;
//     fn deref(&self) -> &ActorContext {
//         &self.context
//     }
// }

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

    impl Actor for Counter {
        fn on_child_failure(&mut self, _reason: String) {}
    }

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

        actor_ref.tell(7);
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
            actor_ref.tell(i);
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
        let ha = std::thread::spawn(move || a.tell(1));
        let hb = std::thread::spawn(move || b.tell(2));
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

    impl Actor for Flaky {
        fn on_child_failure(&mut self, _reason: String) {}
    }

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

        actor_ref.tell(1);
        actor_ref.tell(2); // panics, should not take the actor down
        actor_ref.tell(3);

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

        actor_ref.tell(1);
        actor_ref.tell(2);
        wait_for(&signal, 2);

        actor_ref.stop();
        wait_for(&stop_count, 1);

        // Idempotent: a second stop() must not run on_stop again.
        actor_ref.stop();
        assert_eq!(*stop_count.0.lock().unwrap(), 1);

        // Sends after stop() are dead-lettered, not delivered.
        actor_ref.tell(3);
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

        system.publish(Counted(42));
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
        system.publish(Warning("disk almost full"));
        system.publish(Counted(1));
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

        system.publish(Counted(9));
        wait_for(&signal_a, 1);
        wait_for(&signal_b, 1);

        assert_eq!(*seen_a.lock().unwrap(), vec![9]);
        assert_eq!(*seen_b.lock().unwrap(), vec![9]);
    }

    #[test]
    fn publish_with_no_subscribers_does_not_panic() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        system.publish(Counted(1)); // should just be dropped
    }

    #[test]
    fn has_subscribers_reflects_registration_state() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        assert!(!system.has_subscribers::<Counted>());

        system.subscribe::<Counted>(|_msg, _ctx| {});

        assert!(system.has_subscribers::<Counted>());
        assert!(!system.has_subscribers::<Warning>());
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
            system.publish(Counted(i as i32));
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

        actor_ref.tell(1);
        wait_for(&signal, 1);

        assert_eq!(*seen.lock().unwrap(), vec![std::any::type_name::<i32>()]);
    }
}
