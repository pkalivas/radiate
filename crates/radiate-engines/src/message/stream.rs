use radiate_core::Executor;
use radiate_utils::sentry_id;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, Mutex, RwLock, atomic::AtomicU64},
};

sentry_id!(EventId);

type Callback = Arc<Mutex<dyn FnMut(&dyn Any, &EventCtx) + Send + Sync>>;
type CallbackGroup = Arc<Vec<Callback>>;
type CallbackMap = HashMap<TypeId, CallbackGroup>;

pub trait Event: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Event for T {}

pub trait EventHandler<E>: Send + Sync + 'static {
    fn handle(&mut self, event: &E, ctx: &EventCtx);
}

impl<E, F> EventHandler<E> for F
where
    F: FnMut(&E) + Send + Sync + 'static,
{
    fn handle(&mut self, event: &E, _: &EventCtx) {
        self(event)
    }
}

pub struct EventCtx {
    id: EventId,
    bus: EventStream,
}

impl EventCtx {
    pub fn id(&self) -> &EventId {
        &self.id
    }

    pub fn publish<M: Event>(&self, message: M) {
        self.bus.publish(message);
    }
}

#[derive(Clone, Default)]
pub struct EventStream {
    executor: Arc<Executor>,
    callbacks: Arc<RwLock<CallbackMap>>,
}

impl EventStream {
    pub fn new(executor: Arc<Executor>) -> Self {
        EventStream {
            executor,
            callbacks: Arc::default(),
        }
    }

    pub fn set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor;
    }

    pub fn handler_count<E: Event>(&self) -> usize {
        let type_id = TypeId::of::<E>();
        let subscribers = self.callbacks.read().unwrap();
        match subscribers.get(&type_id) {
            Some(list) => list.len(),
            None => 0,
        }
    }

    pub fn subscribe<M: Event>(&self, mut handler: impl EventHandler<M>) {
        let type_id = TypeId::of::<M>();

        let any_handler = Arc::new(Mutex::new(move |event: &dyn Any, ctx: &EventCtx| {
            if let Some(event) = event.downcast_ref::<M>() {
                handler.handle(event, ctx);
            }
        }));

        let mut subscribers = self.callbacks.write().unwrap();
        let list = subscribers
            .entry(type_id)
            .or_insert_with(|| Arc::new(Vec::new()));
        Arc::make_mut(list).push(any_handler);
    }

    pub fn lazy_publish<M: Event>(&self, f: impl FnOnce() -> M) {
        if self.can_publish::<M>() {
            self.publish(f());
        }
    }

    pub fn publish<M: Event>(&self, message: M) {
        let type_id = TypeId::of::<M>();
        let subs = {
            let subscribers = self.callbacks.read().unwrap();
            match subscribers.get(&type_id) {
                Some(list) => Arc::clone(list),
                None => return,
            }
        };

        let arc_msg = Arc::new(message);
        let id = EventId::new();

        for handler in subs.iter() {
            let cloned_sub = Arc::clone(&handler);
            let message = Arc::clone(&arc_msg);
            let ctx = EventCtx {
                id,
                bus: self.clone(),
            };

            self.executor.submit(move || {
                match cloned_sub.try_lock() {
                    Ok(mut guard) => (*guard)(message.as_ref(), &ctx),
                    Err(std::sync::TryLockError::WouldBlock) => {
                        panic!(
                            "event handler re-entered itself — likely a handler synchronously \
                            re-publishing an event of the same type it's currently handling \
                            under Executor::Serial"
                        );
                    }
                    Err(std::sync::TryLockError::Poisoned(e)) => {
                        // a previous call panicked; recover instead of leaving this
                        // handler permanently dead
                        (*e.into_inner())(message.as_ref(), &ctx);
                    }
                }
            });
        }
    }

    fn can_publish<M: Event>(&self) -> bool {
        self.callbacks
            .read()
            .unwrap()
            .contains_key(&TypeId::of::<M>())
    }
}

impl Debug for EventStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let subscribers = self.callbacks.read().unwrap();
        write!(
            f,
            "EventStream(subscribers={}, executor={:?})",
            subscribers.len(),
            self.executor
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    #[derive(Debug, Clone, PartialEq)]
    struct Ping(u32);

    #[derive(Debug, Clone, PartialEq)]
    struct Pong(u32);

    fn serial_bus() -> EventStream {
        EventStream::new(Arc::new(Executor::Serial))
    }

    #[test]
    fn subscriber_receives_published_events_in_order() {
        let bus = serial_bus();
        let received = Arc::new(Mutex::new(Vec::new()));

        let received_clone = Arc::clone(&received);
        bus.subscribe::<Ping>(move |event: &Ping| {
            received_clone.lock().unwrap().push(event.clone());
        });

        bus.publish(Ping(1));
        bus.publish(Ping(2));

        assert_eq!(*received.lock().unwrap(), vec![Ping(1), Ping(2)]);
    }

    #[test]
    fn multiple_subscribers_all_receive_the_event() {
        let bus = serial_bus();
        let count = Arc::new(AtomicUsize::new(0));

        for _ in 0..3 {
            let count = Arc::clone(&count);
            bus.subscribe::<Ping>(move |_: &Ping| {
                count.fetch_add(1, Ordering::SeqCst);
            });
        }

        bus.publish(Ping(1));

        assert_eq!(count.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn only_subscribers_of_the_matching_type_are_called() {
        let bus = serial_bus();
        let ping_calls = Arc::new(AtomicUsize::new(0));
        let pong_calls = Arc::new(AtomicUsize::new(0));

        let p1 = Arc::clone(&ping_calls);
        bus.subscribe::<Ping>(move |_: &Ping| {
            p1.fetch_add(1, Ordering::SeqCst);
        });

        let p2 = Arc::clone(&pong_calls);
        bus.subscribe::<Pong>(move |_: &Pong| {
            p2.fetch_add(1, Ordering::SeqCst);
        });

        bus.publish(Ping(1));

        assert_eq!(ping_calls.load(Ordering::SeqCst), 1);
        assert_eq!(pong_calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn publish_with_no_subscribers_is_a_no_op() {
        let bus = serial_bus();
        bus.publish(Ping(1)); // must not panic
    }

    #[test]
    fn handler_state_persists_across_separate_publishes() {
        let bus = serial_bus();
        let observed = Arc::new(Mutex::new(Vec::new()));
        let observed_clone = Arc::clone(&observed);

        let mut running_total = 0u32;
        bus.subscribe::<Ping>(move |event: &Ping| {
            running_total += event.0;
            observed_clone.lock().unwrap().push(running_total);
        });

        bus.publish(Ping(1));
        bus.publish(Ping(2));
        bus.publish(Ping(3));

        // 1, 1+2, 1+2+3 — confirms the FnMut's captured state survives
        // between calls rather than being re-created each publish.
        assert_eq!(*observed.lock().unwrap(), vec![1, 3, 6]);
    }

    struct SumHandler {
        total: Arc<AtomicUsize>,
    }

    impl EventHandler<Ping> for SumHandler {
        fn handle(&mut self, event: &Ping, _ctx: &EventCtx) {
            self.total.fetch_add(event.0 as usize, Ordering::SeqCst);
        }
    }

    #[test]
    fn struct_based_handler_implements_event_handler_directly() {
        let bus = serial_bus();
        let total = Arc::new(AtomicUsize::new(0));

        bus.subscribe::<Ping>(SumHandler {
            total: Arc::clone(&total),
        });

        bus.publish(Ping(2));
        bus.publish(Ping(5));

        assert_eq!(total.load(Ordering::SeqCst), 7);
    }

    struct PingRelay;

    impl EventHandler<Ping> for PingRelay {
        fn handle(&mut self, event: &Ping, ctx: &EventCtx) {
            // Different event type than the one we're currently handling —
            // no mutex conflict, unlike re-publishing the same type would be.
            ctx.publish(Pong(event.0 * 10));
        }
    }

    #[test]
    fn handler_can_publish_a_different_event_type_from_within_itself() {
        let bus = serial_bus();
        let pong_received = Arc::new(Mutex::new(None));
        let pong_received_clone = Arc::clone(&pong_received);

        bus.subscribe::<Pong>(move |event: &Pong| {
            *pong_received_clone.lock().unwrap() = Some(event.clone());
        });
        bus.subscribe::<Ping>(PingRelay);

        bus.publish(Ping(4));

        assert_eq!(*pong_received.lock().unwrap(), Some(Pong(40)));
    }

    #[test]
    fn lazy_publish_only_evaluates_message_when_a_subscriber_exists() {
        let bus = serial_bus();
        let evaluated = Arc::new(AtomicUsize::new(0));

        // No Pong subscribers yet: the closure must not run.
        let e1 = Arc::clone(&evaluated);
        bus.lazy_publish::<Pong>(move || {
            e1.fetch_add(1, Ordering::SeqCst);
            Pong(0)
        });
        assert_eq!(evaluated.load(Ordering::SeqCst), 0);

        // Now subscribe — the closure should run this time.
        bus.subscribe::<Pong>(|_: &Pong| {});
        let e2 = Arc::clone(&evaluated);
        bus.lazy_publish::<Pong>(move || {
            e2.fetch_add(1, Ordering::SeqCst);
            Pong(1)
        });
        assert_eq!(evaluated.load(Ordering::SeqCst), 1);
    }

    // Depends on the `try_lock` + poisoned-mutex-recovery patch in `publish`.
    // Without it, the second `publish` call below panics via `.unwrap()`
    // on a poisoned mutex instead of recovering.
    #[test]
    fn handler_panic_does_not_permanently_break_the_subscriber() {
        let bus = serial_bus();
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_clone = Arc::clone(&calls);

        bus.subscribe::<Ping>(move |event: &Ping| {
            calls_clone.fetch_add(1, Ordering::SeqCst);
            if event.0 == 0 {
                panic!("simulated handler failure");
            }
        });

        // Serial mode runs the handler inline, so the panic propagates
        // straight out of `publish` on this thread.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            bus.publish(Ping(0));
        }));
        assert!(result.is_err());

        // The handler's mutex is now poisoned — this must still succeed.
        bus.publish(Ping(5));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn publish_dispatches_asynchronously_on_a_worker_pool() {
        let bus = EventStream::new(Arc::new(Executor::FixedSizedWorkerPool(2)));
        let received: Arc<Mutex<Option<u32>>> = Arc::new(Mutex::new(None));

        let received_clone = Arc::clone(&received);
        bus.subscribe::<Ping>(move |event: &Ping| {
            *received_clone.lock().unwrap() = Some(event.0);
        });

        bus.publish(Ping(7));

        // Dispatch happens on a pool worker, not inline — poll briefly.
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if let Some(value) = *received.lock().unwrap() {
                assert_eq!(value, 7);
                return;
            }
            assert!(Instant::now() < deadline, "handler was not invoked in time");
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    // Documents the known hazard rather than asserting anything: a handler
    // that synchronously re-publishes its own event type under
    // `Executor::Serial` re-enters its own already-locked mutex. With the
    // `try_lock` patch this panics loudly (WouldBlock); without it, it hangs
    // forever. Left `#[ignore]` so it doesn't block a normal test run —
    // run with `cargo test -- --ignored` to see the panic message.
    #[test]
    #[ignore = "demonstrates same-type reentrant publish under Executor::Serial"]
    fn reentrant_same_type_publish_under_serial_is_unsafe() {
        struct SelfRepublisher(u32);

        impl EventHandler<Ping> for SelfRepublisher {
            fn handle(&mut self, event: &Ping, ctx: &EventCtx) {
                if event.0 < self.0 {
                    ctx.publish(Ping(event.0 + 1));
                }
            }
        }

        let bus = serial_bus();
        bus.subscribe::<Ping>(SelfRepublisher(3));
        bus.publish(Ping(0));
    }
}
