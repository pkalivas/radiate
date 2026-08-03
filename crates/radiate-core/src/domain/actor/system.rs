use super::actor::{Actor, AnyActor};
use super::handler::{EventContext, EventHandler};
use super::message::Message;
use crate::Executor;
use crate::ThreadSync;
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

/// Everyone subscribed to one concrete message type. `type_name` is captured
/// once, at first subscription, purely for `Debug` — `TypeId` alone prints
/// as an opaque hash, telling you nothing about which message kind you're
/// looking at.
struct Subscription {
    type_name: &'static str,
    actors: Vec<Arc<dyn AnyActor>>,
}

impl fmt::Debug for Subscription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(self.type_name)
            .field("actors", &self.actors)
            .finish()
    }
}

type ActorRegistry = Arc<Mutex<HashMap<TypeId, Subscription>>>;

/// A small, generic actor system: subscribers are keyed by the concrete
/// message type they registered for, each with its own mailbox. Sending an
/// `M` only ever touches actors that subscribed to `M` — unrelated message
/// types (a different `M2`) live under a different `TypeId` and are never
/// even looked at.
#[derive(Clone)]
pub struct ActorSystem {
    actors: ActorRegistry,
    executor: Arc<Mutex<Arc<Executor>>>,
    sync: Arc<Mutex<ThreadSync>>,
}

impl ActorSystem {
    pub fn new(executor: Arc<Executor>) -> Self {
        ActorSystem::with_sync(executor, ThreadSync::new())
    }

    pub fn with_sync(executor: Arc<Executor>, sync: ThreadSync) -> Self {
        ActorSystem {
            actors: Arc::new(Mutex::new(HashMap::new())),
            executor: Arc::new(Mutex::new(executor)),
            sync: Arc::new(Mutex::new(sync)),
        }
    }

    /// Swap the executor used for future dispatch. Existing subscribers are
    /// untouched — this only changes how `send` schedules drains from this
    /// point on.
    pub fn set_executor(&self, executor: Arc<Executor>) {
        *self.executor.lock().unwrap() = executor;
    }

    /// Swap the `ThreadSync` handed to actors from this point on. Used to
    /// bind an `ActorSystem` built with a placeholder (e.g. during a builder
    /// chain) to the same control primitive the owning engine ends up using,
    /// so `ctx.sync` in any handler and `engine.control()` are the same
    /// object.
    pub fn set_sync(&self, sync: ThreadSync) {
        *self.sync.lock().unwrap() = sync;
    }

    pub fn sync(&self) -> ThreadSync {
        self.sync.lock().unwrap().clone()
    }

    fn current_executor(&self) -> Arc<Executor> {
        Arc::clone(&self.executor.lock().unwrap())
    }

    fn current_context(&self) -> EventContext {
        EventContext {
            sync: self.sync(),
            system: self.clone(),
        }
    }

    /// Register a handler for message type `M`. Takes `&self` — subscribing
    /// is just inserting into the actor registry under its lock, no `&mut
    /// ActorSystem` needed, so an `ActorSystem` can be freely shared (e.g.
    /// via `Clone`) and subscribed to from multiple places without
    /// coordination.
    pub fn subscribe<M, H>(&self, handler: H)
    where
        M: Message,
        H: EventHandler<M> + 'static,
    {
        let actor: Arc<dyn AnyActor> = Actor::new(Box::new(handler));
        self.actors
            .lock()
            .unwrap()
            .entry(TypeId::of::<M>())
            .or_insert_with(|| Subscription {
                type_name: std::any::type_name::<M>(),
                actors: Vec::new(),
            })
            .actors
            .push(actor);
    }

    /// Cheap, payload-free check: does anyone care about `M` at all? Callers
    /// that construct an expensive `M` should check this first and skip
    /// construction entirely when nobody's listening — `send` alone can't
    /// help with that since by the time it's called, `M` already exists.
    pub fn has_subscribers<M: Message>(&self) -> bool {
        self.actors
            .lock()
            .unwrap()
            .get(&TypeId::of::<M>())
            .is_some_and(|sub| !sub.actors.is_empty())
    }

    /// Send a message. Only actors subscribed to `TypeId::of::<M>()` are
    /// touched — if nobody's listening for this kind, this is just a map
    /// lookup, no payload cloning happens.
    ///
    /// The registry lock is dropped before any actor is told about the
    /// message — under a `Serial` executor, `tell` drains the actor's
    /// mailbox (and so runs the handler) inline, on this same call stack.
    /// A handler that calls `EventContext::send`/`has_subscribers` back
    /// into this same `ActorSystem` would then try to re-lock a `Mutex`
    /// this thread already holds, which deadlocks instead of blocking
    /// briefly (`std::sync::Mutex` isn't reentrant).
    /// Send a message. Only actors subscribed to `TypeId::of::<M>()` are
    /// touched — if nobody's listening for this kind, this is just a map
    /// lookup, no payload cloning happens.
    pub fn send<M: Message + Clone>(&self, message: M) {
        let actors = {
            let registry = self.actors.lock().unwrap();
            match registry.get(&TypeId::of::<M>()) {
                Some(sub) => sub
                    .actors
                    .iter()
                    .filter_map(|actor| Arc::clone(actor).as_any_arc().downcast::<Actor<M>>().ok())
                    .collect::<Vec<_>>(),
                None => return,
            }
        };

        let executor = self.current_executor();
        let ctx = self.current_context();
        for actor in actors.iter() {
            actor.tell(message.clone(), ctx.clone(), &executor);
        }
    }
}

impl Default for ActorSystem {
    fn default() -> Self {
        ActorSystem::new(Arc::new(Executor::default()))
    }
}

impl fmt::Debug for ActorSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let actors = self.actors.lock().unwrap();
        f.debug_struct("ActorSystem")
            .field("subscriptions", &actors.values().collect::<Vec<_>>())
            .field("executor", &self.executor.lock().unwrap())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Condvar;
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
        fn handle(&mut self, message: Counted, _ctx: &EventContext) {
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
        let bus = ActorSystem::default();
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
        let bus = ActorSystem::default();
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
        let bus = ActorSystem::default();
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
        let bus = ActorSystem::new(Arc::new(Executor::FixedSizedWorkerPool(4)));
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
        let bus = ActorSystem::default();
        bus.send(Counted(1));
    }

    #[test]
    fn closures_work_as_handlers() {
        let bus = ActorSystem::default();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let seen2 = Arc::clone(&seen);
        let signal2 = Arc::clone(&signal);
        bus.subscribe::<Warning, _>(move |w: Warning, _ctx: &EventContext| {
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
        let bus = ActorSystem::default();
        assert!(!bus.has_subscribers::<Counted>());

        bus.subscribe::<Counted, _>(|_msg: Counted, _ctx: &EventContext| {});
        assert!(bus.has_subscribers::<Counted>());
        assert!(!bus.has_subscribers::<Warning>());
    }

    #[test]
    fn handler_receives_shared_thread_sync_via_context() {
        let sync = ThreadSync::new();
        let bus = ActorSystem::with_sync(Arc::new(Executor::default()), sync.clone());
        let signal = Arc::new((Mutex::new(0), Condvar::new()));
        let signal_clone = Arc::clone(&signal);

        assert!(!sync.is_stopped());

        bus.subscribe::<Counted, _>(move |_msg: Counted, ctx: &EventContext| {
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
        let bus = ActorSystem::default();
        let warnings_seen = Arc::new(Mutex::new(0));
        let escalated = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let escalated2 = Arc::clone(&escalated);
        let signal2 = Arc::clone(&signal);
        bus.subscribe::<Warning, _>(move |w: Warning, ctx: &EventContext| {
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
        bus.subscribe::<Counted, _>(move |_msg: Counted, _ctx: &EventContext| {
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
        let bus = ActorSystem::default();
        let seen = Arc::new(Mutex::new(false));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let seen2 = Arc::clone(&seen);
        let signal2 = Arc::clone(&signal);
        bus.subscribe::<Counted, _>(move |_msg: Counted, ctx: &EventContext| {
            *seen2.lock().unwrap() = ctx.has_subscribers::<Warning>();
            let (n, cv) = &*signal2;
            *n.lock().unwrap() += 1;
            cv.notify_all();
        });

        // Nobody subscribed to Warning yet.
        bus.send(Counted(1));
        wait_for(&signal, 1);
        assert!(!*seen.lock().unwrap());

        bus.subscribe::<Warning, _>(|_w: Warning, _ctx: &EventContext| {});

        bus.send(Counted(2));
        wait_for(&signal, 2);
        assert!(*seen.lock().unwrap());
    }

    #[test]
    fn set_sync_rebinds_context_for_future_sends() {
        let bus = ActorSystem::default();
        let seen_stopped = Arc::new(Mutex::new(false));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let seen_stopped2 = Arc::clone(&seen_stopped);
        let signal2 = Arc::clone(&signal);
        bus.subscribe::<Counted, _>(move |_msg: Counted, ctx: &EventContext| {
            *seen_stopped2.lock().unwrap() = ctx.sync.is_stopped();
            let (n, cv) = &*signal2;
            *n.lock().unwrap() += 1;
            cv.notify_all();
        });

        let real_sync = ThreadSync::new();
        real_sync.stop();
        bus.set_sync(real_sync);

        bus.send(Counted(1));
        wait_for(&signal, 1);

        assert!(*seen_stopped.lock().unwrap());
    }
}
