use crate::Executor;
use crate::domain::sync::control::ThreadSync;
use std::any::{Any, TypeId};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Anything that can ride the bus. Blanket-implemented — the only real
/// requirement is being safe to hand across the `Executor`'s worker threads.
pub trait Message: Send + Sync + 'static {}
impl<M: Send + Sync + 'static> Message for M {}

/// Handed to every actor alongside the message it's processing. Carries the
/// same `ThreadSync` the owning engine (or whatever else set up this
/// `ActorSystem`) uses for pause/stop/step — so any actor can act back on
/// the thing it's observing, not just read it.
#[derive(Clone)]
pub struct ActorContext {
    pub sync: ThreadSync,
}

pub trait EventHandler<M>: Send + Sync {
    fn handle(&mut self, message: M, ctx: &ActorContext);
}

impl<M, F> EventHandler<M> for F
where
    F: FnMut(M, &ActorContext) + Send + Sync,
{
    fn handle(&mut self, message: M, ctx: &ActorContext) {
        self(message, ctx)
    }
}

/// A cheaply-clonable message envelope: wraps `D` in an `Arc` so fanning a
/// message out to many subscribed actors clones a pointer per actor, not the
/// payload itself. Most concrete message types on the bus should be a type
/// alias over this rather than hand-rolling their own `Arc` wrapper.
pub struct Envelope<D>(Arc<D>);

impl<D> Envelope<D> {
    pub fn new(data: D) -> Self {
        Envelope(Arc::new(data))
    }
}

impl<D> Clone for Envelope<D> {
    fn clone(&self) -> Self {
        Envelope(Arc::clone(&self.0))
    }
}

impl<D> std::ops::Deref for Envelope<D> {
    type Target = D;

    fn deref(&self) -> &D {
        &self.0
    }
}

/// A single subscriber's mailbox. `tell` enqueues (message, context) pairs
/// and, if nobody is currently draining this actor, schedules a drain on the
/// executor. `scheduled` guarantees at most one in-flight drain per actor,
/// which is what gives every actor FIFO delivery and non-concurrent handling
/// regardless of how many worker threads the executor itself has.
///
/// The context is captured per-message at `tell` time (same as the
/// executor), not looked up fresh at drain time — `ActorSystem::set_sync`
/// only ever happens once, early, before real traffic starts, so this is
/// just the simplest thing that's still correct.
struct Actor<M: Message> {
    handler: Mutex<Box<dyn EventHandler<M>>>,
    mailbox: Mutex<VecDeque<(M, ActorContext)>>,
    scheduled: AtomicBool,
}

impl<M: Message> Actor<M> {
    fn new(handler: Box<dyn EventHandler<M>>) -> Arc<Self> {
        Arc::new(Actor {
            handler: Mutex::new(handler),
            mailbox: Mutex::new(VecDeque::new()),
            scheduled: AtomicBool::new(false),
        })
    }

    fn tell(self: &Arc<Self>, message: M, ctx: ActorContext, executor: &Executor) {
        self.mailbox.lock().unwrap().push_back((message, ctx));

        if self
            .scheduled
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            let this = Arc::clone(self);
            executor.submit(move || this.drain());
        }
    }

    fn drain(self: Arc<Self>) {
        loop {
            let next = self.mailbox.lock().unwrap().pop_front();
            match next {
                Some((message, ctx)) => self.handler.lock().unwrap().handle(message, &ctx),
                None => {
                    self.scheduled.store(false, Ordering::Release);

                    // Something may have been pushed between the pop above
                    // returning `None` and clearing `scheduled`. Re-claim the
                    // slot and keep draining if so, otherwise we're done.
                    let more_arrived = !self.mailbox.lock().unwrap().is_empty();
                    if !more_arrived
                        || self
                            .scheduled
                            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                            .is_err()
                    {
                        return;
                    }
                }
            }
        }
    }
}

type ActorRegistry = Arc<Mutex<HashMap<TypeId, Vec<Arc<dyn Any + Send + Sync>>>>>;

/// A small, generic actor system: subscribers are keyed by the concrete
/// message type they registered for, each with its own mailbox. Sending an
/// `M` only ever touches actors that subscribed to `M` — unrelated message
/// types (a different `M2`) live under a different `TypeId` and are never
/// even looked at.
#[derive(Clone)]
pub struct ActorSystem {
    actors: ActorRegistry,
    // Mutex, not a plain Arc<Executor>/ThreadSync: callers often need to
    // collect subscribers before the real executor/sync are known (e.g. a
    // builder that binds them last), and swapping either shouldn't require
    // rebuilding the actor registry.
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

    fn current_context(&self) -> ActorContext {
        ActorContext { sync: self.sync() }
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
        let actor: Arc<dyn Any + Send + Sync> = Actor::new(Box::new(handler));
        self.actors
            .lock()
            .unwrap()
            .entry(TypeId::of::<M>())
            .or_default()
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
            .is_some_and(|subs| !subs.is_empty())
    }

    /// Send a message. Only actors subscribed to `TypeId::of::<M>()` are
    /// touched — if nobody's listening for this kind, this is just a map
    /// lookup, no payload cloning happens.
    pub fn send<M: Message + Clone>(&self, message: M) {
        let actors = self.actors.lock().unwrap();
        let Some(subscribers) = actors.get(&TypeId::of::<M>()) else {
            return;
        };

        let executor = self.current_executor();
        let ctx = self.current_context();
        for erased in subscribers {
            if let Ok(actor) = Arc::clone(erased).downcast::<Actor<M>>() {
                actor.tell(message.clone(), ctx.clone(), &executor);
            }
        }
    }
}

impl Default for ActorSystem {
    fn default() -> Self {
        ActorSystem::new(Arc::new(Executor::default()))
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
        fn handle(&mut self, message: Counted, _ctx: &ActorContext) {
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
        bus.subscribe::<Warning, _>(move |w: Warning, _ctx: &ActorContext| {
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

        bus.subscribe::<Counted, _>(|_msg: Counted, _ctx: &ActorContext| {});
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

        bus.subscribe::<Counted, _>(move |_msg: Counted, ctx: &ActorContext| {
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
    fn set_sync_rebinds_context_for_future_sends() {
        let bus = ActorSystem::default();
        let seen_stopped = Arc::new(Mutex::new(false));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let seen_stopped2 = Arc::clone(&seen_stopped);
        let signal2 = Arc::clone(&signal);
        bus.subscribe::<Counted, _>(move |_msg: Counted, ctx: &ActorContext| {
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
