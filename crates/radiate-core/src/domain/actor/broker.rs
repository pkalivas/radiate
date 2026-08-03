use super::actor::Actor;
use super::handler::EventHandler;
use super::message::Message;
use crate::{
    Envelope, ThreadSync,
    actor::{
        message::EventId,
        subscriber::{AnySubscription, Subscription},
    },
};
use crate::{EventContext, Executor};
use std::fmt;
use std::sync::{Arc, RwLock};
use std::{any::TypeId, fmt::Debug};
use std::{collections::HashMap, marker::PhantomData};

type ActorRegistry = Arc<RwLock<HashMap<TypeId, Box<dyn AnySubscription>>>>;

/// System-wide snapshot returned by [`ActorSystem::stats`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ActorSystemStats {
    /// Distinct message kinds with at least one subscriber.
    pub subscriptions: usize,
    /// Total actors across all subscriptions (a kind with 3 subscribers
    /// counts 3 here).
    pub actors: usize,
    /// Messages currently sitting in a mailbox, summed across every actor.
    pub queued: usize,
    /// Messages processed since each actor was created, summed across
    /// every actor. Monotonically increasing per actor, so this only ever
    /// grows (or resets to 0 if actors are re-subscribed from scratch).
    pub processed: u64,
}

pub struct SubscriptionBuilder<'a, M: Message> {
    pub(crate) system: &'a MessageBroker,
    pub(crate) _marker: PhantomData<M>,
}

impl<M: Message + Debug> SubscriptionBuilder<'_, M> {
    pub fn handle<H>(self, handler: H)
    where
        H: EventHandler<M> + 'static,
    {
        self.system.subscribe::<M, H>(handler);
    }
}

/// A small, generic actor system/message broker: subscribers are keyed by the concrete
/// message type they registered for, each with its own mailbox. Sending an
/// `M` only ever touches actors that subscribed to `M` — unrelated message
/// types (a different `M2`) live under a different `TypeId` and are never
/// even looked at.
#[derive(Clone)]
pub struct MessageBroker {
    actors: ActorRegistry,
    executor: Arc<Executor>,
    sync: ThreadSync,
}

impl MessageBroker {
    pub fn new(executor: Arc<Executor>) -> Self {
        MessageBroker {
            actors: Arc::new(RwLock::new(HashMap::new())),
            executor,
            sync: ThreadSync::new(),
        }
    }

    pub fn subscribers(&self) -> ActorRegistry {
        Arc::clone(&self.actors)
    }

    /// Cheap, payload-free check: does anyone care about `M` at all? Callers
    /// that construct an expensive `M` should check this first and skip
    /// construction entirely when nobody's listening — `send` alone can't
    /// help with that since by the time it's called, `M` already exists.
    pub fn has_subscribers<M: Message>(&self) -> bool {
        self.actors
            .read()
            .unwrap()
            .get(&TypeId::of::<M>())
            .is_some_and(|sub| sub.num_actors() > 0)
    }

    pub fn on<M: Message + Debug>(&self) -> SubscriptionBuilder<'_, M> {
        SubscriptionBuilder {
            system: self,
            _marker: PhantomData,
        }
    }

    /// Register a handler for message type `M`. Takes `&self` — subscribing
    /// is just inserting into the actor registry under its lock, no `&mut
    /// ActorSystem` needed, so an `ActorSystem` can be freely shared (e.g.
    /// via `Clone`) and subscribed to from multiple places without
    /// coordination.
    pub fn subscribe<M, H>(&self, handler: H)
    where
        M: Message + Debug,
        H: EventHandler<M> + 'static,
    {
        let mut registry = self.actors.write().unwrap();

        let sub = registry
            .entry(TypeId::of::<M>())
            .or_insert_with(|| {
                Box::new(Subscription::<M> { actors: Vec::new() }) as Box<dyn AnySubscription>
            })
            .as_any_mut()
            .downcast_mut::<Subscription<M>>()
            .unwrap();

        sub.actors.push(Actor::new(Box::new(handler)));
    }

    /// A system-wide health snapshot: how many message kinds have at least
    /// one subscriber, how many actors that spans, how many messages are
    /// currently sitting in a mailbox waiting to be drained, and how many
    /// have been processed since each actor was created. Aggregated rather
    /// than broken out per message kind — `Subscription`'s `type_name` comes
    /// from `std::any::type_name::<M>()`, which is fine for `Debug` but not
    /// something worth turning into a metric name (full module paths,
    /// mangled generics).
    pub fn stats(&self) -> ActorSystemStats {
        let registry = self.actors.read().unwrap();

        let mut actors = 0usize;
        let mut queued = 0usize;
        let mut processed = 0u64;

        for sub in registry.values() {
            actors += sub.num_actors();
            queued += sub.queued();
            processed += sub.processed();
        }

        ActorSystemStats {
            subscriptions: registry.len(),
            actors,
            queued,
            processed,
        }
    }

    /// Send a message. Only actors subscribed to `TypeId::of::<M>()` are
    /// touched — if nobody's listening for this kind, this is just a map
    /// lookup, no payload cloning happens.
    ///
    /// The registry lock is dropped before any actor is told about the
    /// message — under a `Serial` executor, `tell` drains the actor's
    /// mailbox (and so runs the handler) inline, on this same call stack.
    /// A handler that calls `EventContext::send`/`has_subscribers` back
    /// into this same `ActorSystem` would then try to re-lock a lock this
    /// thread already holds, which deadlocks instead of blocking briefly
    /// (neither `Mutex` nor `RwLock` in `std` are reentrant).
    #[inline]
    pub fn send<M: Message>(&self, message: M) {
        let ctx = EventContext {
            sync: self.sync.clone(),
            system: self.clone(),
            id: EventId::new(),
        };
        let envelope = Envelope::new(message);
        self.send_internal(envelope, ctx);
    }

    #[inline]
    pub fn lazy_send<M: Message>(&self, func: impl FnOnce() -> M) {
        if self.has_subscribers::<M>() {
            let message = func();
            self.send(message);
        }
    }

    #[inline]
    pub fn trace_send<M: Message>(&self, message: M, id: EventId) {
        let ctx = EventContext {
            sync: self.sync.clone(),
            system: self.clone(),
            id,
        };

        let envelope = Envelope::new(message);
        self.send_internal(envelope, ctx);
    }

    #[inline]
    fn send_internal<M: Message>(&self, envelope: Envelope<M>, ctx: EventContext) {
        let executor = Arc::clone(&self.executor);
        let registry = self.actors.read().unwrap();

        if let Some(sub) = registry.get(&TypeId::of::<M>()) {
            sub.send(&envelope, ctx, &executor);
        }
    }
}

impl From<(Arc<Executor>, ThreadSync)> for MessageBroker {
    fn from((executor, sync): (Arc<Executor>, ThreadSync)) -> Self {
        MessageBroker {
            actors: Arc::new(RwLock::new(HashMap::new())),
            executor,
            sync,
        }
    }
}

impl From<(Arc<Executor>, ThreadSync, ActorRegistry)> for MessageBroker {
    fn from((executor, sync, actors): (Arc<Executor>, ThreadSync, ActorRegistry)) -> Self {
        MessageBroker {
            executor,
            sync,
            actors,
        }
    }
}

impl Default for MessageBroker {
    fn default() -> Self {
        MessageBroker::new(Arc::new(Executor::default()))
    }
}

impl fmt::Debug for MessageBroker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let actors = self.actors.read().unwrap();
        f.debug_struct("ActorSystem")
            .field("subscriptions", &actors.values().collect::<Vec<_>>())
            .field("executor", &self.executor)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Condvar, Mutex};
    use std::time::Duration;

    #[derive(Clone, Debug, PartialEq)]
    struct Counted(i32);

    #[derive(Clone, Debug, PartialEq)]
    struct Warning(&'static str);

    struct Recorder {
        seen: Arc<Mutex<Vec<i32>>>,
        signal: Arc<(Mutex<usize>, Condvar)>,
    }

    impl EventHandler<Counted> for Recorder {
        fn handle(&mut self, message: &Counted, _ctx: &EventContext) {
            self.seen.lock().unwrap().push(message.0);
            let (count, cv) = &*self.signal;
            *count.lock().unwrap() += 1;
            cv.notify_all();
        }
    }

    fn wait_for(signal: &Arc<(Mutex<usize>, Condvar)>, target: usize) {
        let (lock, cv) = &**signal;
        let mut count = lock.lock().unwrap();
        while *count < target {
            let (guard, timeout) = cv.wait_timeout(count, Duration::from_secs(2)).unwrap();
            count = guard;
            if timeout.timed_out() && *count < target {
                panic!("timed out waiting for {target} messages, saw {}", *count);
            }
        }
    }

    #[test]
    fn subscribe_and_publish_delivers_message() {
        let bus = MessageBroker::default();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        bus.subscribe::<Counted, _>(Recorder {
            seen: Arc::clone(&seen),
            signal: Arc::clone(&signal),
        });

        bus.send(Counted(42));
        wait_for(&signal, 1);

        assert_eq!(*seen.lock().unwrap(), vec![42]);
    }

    #[test]
    fn unrelated_message_types_do_not_cross_wires() {
        let bus = MessageBroker::default();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        bus.subscribe::<Counted, _>(Recorder {
            seen: Arc::clone(&seen),
            signal: Arc::clone(&signal),
        });

        // Nobody subscribed to Warning, so this should be a silent no-op.
        bus.send(Warning("disk almost full"));
        bus.send(Counted(1));
        wait_for(&signal, 1);

        assert_eq!(*seen.lock().unwrap(), vec![1]);
    }

    #[test]
    fn multiple_subscribers_of_same_type_all_receive() {
        let bus = MessageBroker::default();
        let seen_a = Arc::new(Mutex::new(Vec::new()));
        let seen_b = Arc::new(Mutex::new(Vec::new()));
        let signal_a = Arc::new((Mutex::new(0), Condvar::new()));
        let signal_b = Arc::new((Mutex::new(0), Condvar::new()));

        bus.subscribe::<Counted, _>(Recorder {
            seen: Arc::clone(&seen_a),
            signal: Arc::clone(&signal_a),
        });
        bus.subscribe::<Counted, _>(Recorder {
            seen: Arc::clone(&seen_b),
            signal: Arc::clone(&signal_b),
        });

        bus.send(Counted(7));
        wait_for(&signal_a, 1);
        wait_for(&signal_b, 1);

        assert_eq!(*seen_a.lock().unwrap(), vec![7]);
        assert_eq!(*seen_b.lock().unwrap(), vec![7]);
    }

    #[test]
    fn ordering_preserved_per_actor_under_parallel_executor() {
        let bus = MessageBroker::new(Arc::new(Executor::FixedSizedWorkerPool(4)));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        bus.subscribe::<Counted, _>(Recorder {
            seen: Arc::clone(&seen),
            signal: Arc::clone(&signal),
        });

        const N: usize = 500;
        for i in 0..N as i32 {
            bus.send(Counted(i));
        }
        wait_for(&signal, N);

        let expected: Vec<i32> = (0..N as i32).collect();
        assert_eq!(*seen.lock().unwrap(), expected);
    }

    #[test]
    fn publish_with_no_subscribers_does_not_panic() {
        let bus = MessageBroker::default();
        bus.send(Counted(1));
    }

    #[test]
    fn closures_work_as_handlers() {
        let bus = MessageBroker::default();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let seen2 = Arc::clone(&seen);
        let signal2 = Arc::clone(&signal);
        bus.subscribe::<Warning, _>(move |w: &Warning, _ctx: &EventContext| {
            seen2.lock().unwrap().push(w.0);
            let (count, cv) = &*signal2;
            *count.lock().unwrap() += 1;
            cv.notify_all();
        });

        bus.send(Warning("low disk space"));
        wait_for(&signal, 1);

        assert_eq!(*seen.lock().unwrap(), vec!["low disk space"]);
    }

    #[test]
    fn has_subscribers_reflects_registration_state() {
        let bus = MessageBroker::default();
        assert!(!bus.has_subscribers::<Counted>());

        bus.subscribe::<Counted, _>(|_msg: &Counted, _ctx: &EventContext| {});
        assert!(bus.has_subscribers::<Counted>());
        assert!(!bus.has_subscribers::<Warning>());
    }

    #[test]
    fn handler_receives_shared_thread_sync_via_context() {
        let sync = ThreadSync::new();
        let bus = MessageBroker::from((Arc::new(Executor::default()), sync.clone()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));
        let signal_clone = Arc::clone(&signal);

        assert!(!sync.is_stopped());

        bus.subscribe::<Counted, _>(move |_msg: &Counted, ctx: &EventContext| {
            ctx.sync.stop();
            let (n, cv) = &*signal_clone;
            *n.lock().unwrap() += 1;
            cv.notify_all();
        });

        bus.send(Counted(1));
        wait_for(&signal, 1);

        assert!(sync.is_stopped());
    }

    #[test]
    fn handler_can_publish_further_messages_via_context() {
        let bus = MessageBroker::default();
        let warnings_seen = Arc::new(Mutex::new(0));
        let escalated = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let escalated2 = Arc::clone(&escalated);
        let signal2 = Arc::clone(&signal);
        bus.subscribe::<Warning, _>(move |w: &Warning, ctx: &EventContext| {
            escalated2.lock().unwrap().push(w.0);
            let (n, cv) = &*signal2;
            *n.lock().unwrap() += 1;
            cv.notify_all();
            // Escalate: republish as a plain Counted "alert raised" signal
            // on the same system, via the context handed to this handler.
            ctx.send(Counted(1));
        });

        let warnings_seen2 = Arc::clone(&warnings_seen);
        let signal3 = Arc::clone(&signal);
        bus.subscribe::<Counted, _>(move |_msg: &Counted, _ctx: &EventContext| {
            *warnings_seen2.lock().unwrap() += 1;
            let (n, cv) = &*signal3;
            *n.lock().unwrap() += 1;
            cv.notify_all();
        });

        bus.send(Warning("disk almost full"));
        // Both the Warning handler and the Counted handler it triggers
        // signal on the same counter, so wait for both to have fired.
        wait_for(&signal, 2);

        assert_eq!(*escalated.lock().unwrap(), vec!["disk almost full"]);
        assert_eq!(*warnings_seen.lock().unwrap(), 1);
    }

    #[test]
    fn context_has_subscribers_reflects_live_registration_state() {
        let bus = MessageBroker::default();
        let seen = Arc::new(Mutex::new(false));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let seen2 = Arc::clone(&seen);
        let signal2 = Arc::clone(&signal);
        bus.subscribe::<Counted, _>(move |_msg: &Counted, ctx: &EventContext| {
            *seen2.lock().unwrap() = ctx.has_subscribers::<Warning>();
            let (n, cv) = &*signal2;
            *n.lock().unwrap() += 1;
            cv.notify_all();
        });

        // Nobody subscribed to Warning yet.
        bus.send(Counted(1));
        wait_for(&signal, 1);
        assert!(!*seen.lock().unwrap());

        bus.subscribe::<Warning, _>(|_w: &Warning, _ctx: &EventContext| {});

        bus.send(Counted(2));
        wait_for(&signal, 2);
        assert!(*seen.lock().unwrap());
    }

    #[test]
    fn stats_reflects_subscriptions_and_processed_count() {
        let bus = MessageBroker::default();

        let empty = bus.stats();
        assert_eq!(empty.subscriptions, 0);
        assert_eq!(empty.actors, 0);
        assert_eq!(empty.queued, 0);
        assert_eq!(empty.processed, 0);

        let signal = Arc::new((Mutex::new(0), Condvar::new()));
        let signal2 = Arc::clone(&signal);
        bus.subscribe::<Counted, _>(move |_msg: &Counted, _ctx: &EventContext| {
            let (n, cv) = &*signal2;
            *n.lock().unwrap() += 1;
            cv.notify_all();
        });
        bus.subscribe::<Warning, _>(|_w: &Warning, _ctx: &EventContext| {});

        let registered = bus.stats();
        assert_eq!(
            registered.subscriptions, 2,
            "Counted and Warning both registered"
        );
        assert_eq!(registered.actors, 2);
        assert_eq!(registered.queued, 0);
        assert_eq!(registered.processed, 0);

        const N: usize = 5;
        for i in 0..N as i32 {
            bus.send(Counted(i));
        }
        wait_for(&signal, N);

        // Draining is single-flight and batches the whole mailbox under one
        // lock (see `Actor::drain`), so there's no reliable in-between state
        // to observe here — only that everything sent eventually gets
        // processed and nothing is left queued once it has.
        let done = bus.stats();
        assert_eq!(done.queued, 0);
        assert_eq!(done.processed, N as u64);
    }
}
