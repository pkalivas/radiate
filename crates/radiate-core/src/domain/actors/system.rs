use crate::{
    ActorId,
    actors::{
        actor::{Actor, ActorCell, ActorRef, AnyActorRef},
        handler::FnActor,
        message::DeadLetterActor,
        subscriber::{AnySubscriber, SubscriberGroup},
    },
};
use crate::{Envelope, Executor};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Mutex;
use std::sync::{Arc, RwLock, atomic::AtomicBool};

type ActorRegistry<K> = HashMap<K, Box<dyn Any + Send + Sync>>;

#[derive(Clone, Default)]
pub(crate) struct SystemInner {
    executor: Arc<Executor>,
    bus: Arc<DomainBus>,
    by_type: Arc<RwLock<ActorRegistry<TypeId>>>,
    by_name: Arc<RwLock<ActorRegistry<String>>>,
}

#[derive(Clone)]
pub struct ActorContext {
    pub(crate) inner: Arc<SystemInner>,
    pub(crate) parent: Option<AnyActorRef>,
}

impl ActorContext {
    pub fn executor(&self) -> Arc<Executor> {
        Arc::clone(&self.inner.executor)
    }

    pub fn parent(&self) -> Option<AnyActorRef> {
        self.parent.clone()
    }

    pub fn bus(&self) -> Arc<DomainBus> {
        Arc::clone(&self.inner.bus)
    }

    pub fn publish<M: Send + 'static>(&self, message: M) {
        self.inner.bus.publish(message);
    }

    pub fn lazy_publish<M: Send + 'static>(&self, func: impl FnOnce() -> M) {
        if self.has_subscribers::<M>() {
            self.publish(func());
        }
    }

    pub fn has_subscribers<M: Send + 'static>(&self) -> bool {
        self.inner.bus.has_subscribers::<M>()
    }

    /// Silently drops the message if `A` hasn't been spawned. Use when a
    /// missing recipient is a legitimate, expected state.
    pub fn tell<A: Actor + 'static>(&self, message: A::Message) {
        if let Some(r) = self.actor::<A>() {
            r.tell(message);
        }
    }

    pub fn actor<A: Actor + 'static>(&self) -> Option<ActorRef<A::Message>> {
        self.inner
            .by_type
            .read()
            .unwrap()
            .get(&TypeId::of::<A>())
            .and_then(|b| b.downcast_ref::<ActorRef<A::Message>>().cloned())
    }

    pub fn named<M: Send + 'static>(&self, name: &str) -> Option<ActorRef<M>> {
        self.inner
            .by_name
            .read()
            .unwrap()
            .get(name)
            .and_then(|b| b.downcast_ref::<ActorRef<M>>().cloned())
    }
}

#[derive(Clone)]
pub struct ActorSystem {
    context: ActorContext,
}

impl ActorSystem {
    pub fn new(executor: Arc<Executor>) -> Self {
        let inner = Arc::new(SystemInner {
            executor,
            bus: Arc::new(DomainBus::default()),
            by_type: Arc::new(RwLock::new(HashMap::new())),
            by_name: Arc::new(RwLock::new(HashMap::new())),
        });

        ActorSystem {
            context: ActorContext {
                inner,
                parent: None,
            },
        }
    }

    pub fn context(&self) -> ActorContext {
        self.context.clone()
    }

    pub fn listen<A, M>(&self, actor: A)
    where
        A: Actor<Message = M> + 'static,
        M: Send + Sync + Clone + 'static,
    {
        let actor_ref = self.spawn(actor);

        let bridge = FnActor {
            handler: Box::new(move |envelope: Envelope<M>, _ctx: &ActorContext| {
                actor_ref.tell((*envelope).clone());
            }),
        };
        let sub_ref = self.build_ref(bridge);
        self.context.inner.bus.subscribe::<M>(sub_ref);
    }

    pub fn subscribe<M: Send + Sync + 'static>(
        &self,
        mut handler: impl FnMut(&M, &ActorContext) + Send + Sync + 'static,
    ) {
        self.subscribe_with::<M, _>(move |envelope, ctx| handler(&envelope, ctx));
    }

    pub fn spawn<A: Actor + 'static>(&self, actor: A) -> ActorRef<A::Message> {
        let actor_ref = self.build_ref(actor);
        self.context
            .inner
            .by_type
            .write()
            .unwrap()
            .insert(TypeId::of::<A>(), Box::new(actor_ref.clone()));
        actor_ref
    }

    pub fn spawn_named<A: Actor + 'static>(&self, name: &str, actor: A) -> ActorRef<A::Message> {
        let actor_ref = self.spawn(actor);
        self.context
            .inner
            .by_name
            .write()
            .unwrap()
            .insert(name.to_string(), Box::new(actor_ref.clone()));
        actor_ref
    }

    /// Spawns an unregistered `FnActor<Envelope<M>>` wired to call `f` on
    /// each delivery, and subscribes it to the bus for `M`. The shared
    /// plumbing behind both `subscribe` (closure reacts inline) and
    /// `listen` (closure forwards to another actor).
    fn subscribe_with<M, F>(&self, f: F)
    where
        M: Send + Sync + 'static,
        F: FnMut(Envelope<M>, &ActorContext) + Send + Sync + 'static,
    {
        let actor = FnActor {
            handler: Box::new(f),
        };
        let sub_ref = self.build_ref(actor);
        self.context.inner.bus.subscribe::<M>(sub_ref);
    }

    fn build_ref<A: Actor + 'static>(&self, actor: A) -> ActorRef<A::Message> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let cell = Arc::new(ActorCell {
            actor: Arc::new(Mutex::new(actor)),
            receiver: Arc::new(Mutex::new(receiver)),
            scheduled: AtomicBool::new(false),
            context: self.context(),
        });

        ActorRef {
            sender,
            cell,
            executor: self.executor(),
            bus: self.bus(),
            actor_id: ActorId::new(),
        }
    }
}

impl From<(Arc<Executor>, Arc<DomainBus>)> for ActorSystem {
    fn from((executor, bus): (Arc<Executor>, Arc<DomainBus>)) -> Self {
        let inner = Arc::new(SystemInner {
            executor,
            bus,
            by_type: Arc::new(RwLock::new(HashMap::new())),
            by_name: Arc::new(RwLock::new(HashMap::new())),
        });

        ActorSystem {
            context: ActorContext {
                inner,
                parent: None,
            },
        }
    }
}

impl Deref for ActorSystem {
    type Target = ActorContext;
    fn deref(&self) -> &ActorContext {
        &self.context
    }
}

impl Default for ActorSystem {
    fn default() -> Self {
        ActorSystem::new(Arc::new(Executor::default()))
    }
}

#[derive(Default)]
pub struct DomainBus {
    subscribers: RwLock<HashMap<TypeId, Box<dyn AnySubscriber>>>,
}

impl DomainBus {
    pub fn subscribe<M: Send + Sync + 'static>(&self, actor_ref: ActorRef<Envelope<M>>) {
        let mut registry = self.subscribers.write().unwrap();
        let group = registry
            .entry(TypeId::of::<M>())
            .or_insert_with(|| {
                Box::new(SubscriberGroup::<M> {
                    handles: Vec::new(),
                }) as Box<dyn AnySubscriber>
            })
            .as_any_mut()
            .downcast_mut::<SubscriberGroup<M>>()
            .expect("TypeId key always matches SubscriberGroup<M> by construction");
        group.handles.push(actor_ref);
    }

    pub fn publish<M: Send + 'static>(&self, message: M) {
        let registry = self.subscribers.read().unwrap();
        if let Some(group) = registry.get(&TypeId::of::<M>()) {
            group.dispatch(&Envelope::new(message));
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
        type Message = i32;

        fn receive(&mut self, message: i32, _ctx: &ActorContext) {
            self.seen.lock().unwrap().push(message);
            let (count, cv) = &*self.signal;
            *count.lock().unwrap() += 1;
            cv.notify_all();
        }

        fn on_child_failure(&mut self, _reason: String) {}
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
        type Message = i32;

        fn receive(&mut self, message: i32, _ctx: &ActorContext) {
            if message == 2 {
                panic!("boom");
            }
            self.seen.lock().unwrap().push(message);
            let (count, cv) = &*self.signal;
            *count.lock().unwrap() += 1;
            cv.notify_all();
        }

        fn on_child_failure(&mut self, _reason: String) {}
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

    #[test]
    fn erased_ref_with_no_fail_hook_is_a_safe_no_op() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let actor_ref = system.spawn(Counter {
            seen: Arc::clone(&seen),
            signal: Arc::clone(&signal),
        });

        let any_ref = actor_ref.erased();
        any_ref.report_child_failure("shouldn't panic".to_string());
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

// pub fn spawn<A: Actor + 'static>(&self, actor: A) -> ActorRef<A::Message> {
//     let (sender, receiver) = std::sync::mpsc::channel();
//     let cell = Arc::new(ActorCell {
//         actor: Arc::new(Mutex::new(actor)),
//         receiver: Arc::new(Mutex::new(receiver)),
//         scheduled: AtomicBool::new(false),
//         context: self.context(),
//     });

//     let actor_ref = ActorRef {
//         sender,
//         cell,
//         executor: self.executor(),
//         bus: self.bus(),
//         actor_id: ActorId::new(),
//     };

//     self.context
//         .inner
//         .by_type
//         .write()
//         .unwrap()
//         .insert(TypeId::of::<A>(), Box::new(actor_ref.clone()));

//     actor_ref
// }

// pub fn spawn_named<A: Actor + 'static>(&self, name: &str, actor: A) -> ActorRef<A::Message> {
//     let actor_ref = self.spawn(actor);

//     self.context
//         .inner
//         .by_name
//         .write()
//         .unwrap()
//         .insert(name.to_string(), Box::new(actor_ref.clone()));

//     actor_ref
// }

// /// Same as `spawn`, but skips `by_type` registration — used by
// /// `observe` so observer actors are unreachable via `actor::<A>()`.
// fn spawn_unregistered<A: Actor + 'static>(&self, actor: A) -> ActorRef<A::Message> {
//     let (sender, receiver) = std::sync::mpsc::channel();
//     let cell = Arc::new(ActorCell {
//         actor: Arc::new(Mutex::new(actor)),
//         receiver: Arc::new(Mutex::new(receiver)),
//         scheduled: AtomicBool::new(false),
//         context: self.context(),
//     });

//     ActorRef {
//         sender,
//         cell,
//         executor: self.executor(),
//         bus: Arc::clone(&self.context.inner.bus),
//         actor_id: ActorId::new(),
//     }
// }

// pub fn subscribe<M: Send + Sync + 'static>(
//     &self,
//     mut handler: impl FnMut(&M, &ActorContext) + Send + Sync + 'static,
// ) {
//     let actor = FnActor {
//         handler: Box::new(move |message: Envelope<M>, ctx: &ActorContext| {
//             handler(&message, ctx);
//         }),
//     };

//     let sub_ref = self.spawn(actor);
//     self.context.inner.bus.subscribe::<M>(sub_ref);
// }
