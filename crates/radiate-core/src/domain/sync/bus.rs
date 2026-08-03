use crate::Executor;
use std::any::{Any, TypeId};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Anything that can ride the bus. Blanket-implemented — the only real
/// requirement is being safe to hand across the `Executor`'s worker threads.
pub trait Message: Send + Sync + 'static {}
impl<M: Send + Sync + 'static> Message for M {}

pub trait EventHandler<M>: Send + Sync {
    fn handle(&mut self, message: M);
}

impl<M, F> EventHandler<M> for F
where
    F: FnMut(M) + Send + Sync,
{
    fn handle(&mut self, message: M) {
        self(message)
    }
}

/// A single subscriber's mailbox. `tell` enqueues and, if nobody is
/// currently draining this actor, schedules a drain on the executor.
/// `scheduled` guarantees at most one in-flight drain per actor, which is
/// what gives every actor FIFO delivery and non-concurrent handling
/// regardless of how many worker threads the executor itself has.
struct Actor<M: Message> {
    handler: Mutex<Box<dyn EventHandler<M>>>,
    mailbox: Mutex<VecDeque<M>>,
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

    fn tell(self: &Arc<Self>, message: M, executor: &Executor) {
        self.mailbox.lock().unwrap().push_back(message);

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
                Some(message) => self.handler.lock().unwrap().handle(message),
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
/// message type they registered for, each with its own mailbox. Publishing a
/// `M` only ever touches actors that subscribed to `M` — unrelated message
/// types (a different `M2`) live under a different `TypeId` and are never
/// even looked at.
#[derive(Clone)]
pub struct Bus {
    actors: ActorRegistry,
    executor: Arc<Executor>,
}

impl Bus {
    pub fn new(executor: Arc<Executor>) -> Self {
        Bus {
            actors: Arc::new(Mutex::new(HashMap::new())),
            executor,
        }
    }

    /// Register a handler for message type `M`. Takes `&self` — subscribing
    /// is just inserting into the actor registry under its lock, no `&mut
    /// Bus` needed, so a `Bus` can be freely shared (e.g. via `Arc<Bus>` or
    /// `Clone`) and subscribed to from multiple places without coordination.
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

    /// Publish a message. Only actors subscribed to `TypeId::of::<M>()` are
    /// touched — if nobody's listening for this kind, this is just a map
    /// lookup, no payload cloning happens.
    pub fn publish<M: Message + Clone>(&self, message: M) {
        let actors = self.actors.lock().unwrap();
        let Some(subscribers) = actors.get(&TypeId::of::<M>()) else {
            return;
        };

        for erased in subscribers {
            if let Ok(actor) = Arc::clone(erased).downcast::<Actor<M>>() {
                actor.tell(message.clone(), &self.executor);
            }
        }
    }
}

impl Default for Bus {
    fn default() -> Self {
        Bus::new(Arc::new(Executor::default()))
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
        fn handle(&mut self, message: Counted) {
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
        let bus = Bus::default();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        bus.subscribe::<Counted, _>(Recorder {
            seen: Arc::clone(&seen),
            signal: Arc::clone(&signal),
        });

        bus.publish(Counted(42));
        wait_for(&signal, 1);

        assert_eq!(*seen.lock().unwrap(), vec![42]);
    }

    #[test]
    fn unrelated_message_types_do_not_cross_wires() {
        let bus = Bus::default();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        bus.subscribe::<Counted, _>(Recorder {
            seen: Arc::clone(&seen),
            signal: Arc::clone(&signal),
        });

        // Nobody subscribed to Warning, so this should be a silent no-op.
        bus.publish(Warning("disk almost full"));
        bus.publish(Counted(1));
        wait_for(&signal, 1);

        assert_eq!(*seen.lock().unwrap(), vec![1]);
    }

    #[test]
    fn multiple_subscribers_of_same_type_all_receive() {
        let bus = Bus::default();
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

        bus.publish(Counted(7));
        wait_for(&signal_a, 1);
        wait_for(&signal_b, 1);

        assert_eq!(*seen_a.lock().unwrap(), vec![7]);
        assert_eq!(*seen_b.lock().unwrap(), vec![7]);
    }

    #[test]
    fn ordering_preserved_per_actor_under_parallel_executor() {
        let bus = Bus::new(Arc::new(Executor::FixedSizedWorkerPool(4)));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        bus.subscribe::<Counted, _>(Recorder {
            seen: Arc::clone(&seen),
            signal: Arc::clone(&signal),
        });

        const N: usize = 500;
        for i in 0..N as i32 {
            bus.publish(Counted(i));
        }
        wait_for(&signal, N);

        let expected: Vec<i32> = (0..N as i32).collect();
        assert_eq!(*seen.lock().unwrap(), expected);
    }

    #[test]
    fn publish_with_no_subscribers_does_not_panic() {
        let bus = Bus::default();
        bus.publish(Counted(1));
    }

    #[test]
    fn closures_work_as_handlers() {
        let bus = Bus::default();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let seen2 = Arc::clone(&seen);
        let signal2 = Arc::clone(&signal);
        bus.subscribe::<Warning, _>(move |w: Warning| {
            seen2.lock().unwrap().push(w.0);
            let (count, cv) = &*signal2;
            *count.lock().unwrap() += 1;
            cv.notify_all();
        });

        bus.publish(Warning("low disk space"));
        wait_for(&signal, 1);

        assert_eq!(*seen.lock().unwrap(), vec!["low disk space"]);
    }
}
