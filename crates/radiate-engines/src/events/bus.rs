use crate::events::message::*;
use radiate_core::{
    ActorSystem, Chromosome, Envelope, EventHandler, Executor, Message, ThreadSync,
};
use std::marker::PhantomData;
use std::sync::Arc;

/// An engine-specific facade over the generic `radiate_core::ActorSystem`.
/// Every kind of engine event (`on_start`, `on_epoch_complete`, ...) is just
/// a `subscribe::<ConcreteType, _>` on the underlying system — no bespoke
/// trait or adapter layer needed, `radiate_core::EventHandler<M>` already
/// does the job once each kind is its own real type.
#[derive(Clone)]
pub struct EventBus<T> {
    core: ActorSystem,
    _marker: PhantomData<T>,
}

impl<T> EventBus<T> {
    pub fn new(executor: Arc<Executor>, sync: ThreadSync) -> Self {
        EventBus {
            core: ActorSystem::with_sync(executor, sync),
            _marker: PhantomData,
        }
    }

    pub fn subscribe<M, H>(&self, handler: H)
    where
        M: Message + 'static,
        H: EventHandler<M> + 'static,
    {
        self.core.subscribe(handler);
    }

    /// The forward-facing escape hatch for message types this engine
    /// doesn't know about (a custom `Warning`, a `LogMessage`, ...) — rides
    /// the same `ActorSystem` as `Started`/`EpochCompleted`/etc. Registration
    /// happens on the raw `ActorSystem` during the builder chain (see
    /// `GeneticEngineBuilder::subscribe_typed`); this is purely for whoever
    /// ends up owning publishing `M` once it's built.
    pub fn send<M: Message + Clone>(&self, message: M) {
        self.core.send(message);
    }

    pub fn publish<C>(&self, message: EngineMessage<C, T>)
    where
        C: Chromosome,
        T: Clone + Send + Sync + 'static,
    {
        match message {
            EngineMessage::Start(_ctx) => self.dispatch(|| StartedData, EngineEvent::Started),
            EngineMessage::Stop(ctx) => self.dispatch(
                || StoppedData {
                    index: ctx.index,
                    best: ctx.best.clone(),
                    metrics: ctx.metrics.clone(),
                    score: ctx.score.clone().unwrap_or_default(),
                },
                EngineEvent::Stopped,
            ),
            EngineMessage::EpochStart(ctx) => self.dispatch(
                || EpochStartedData { index: ctx.index },
                EngineEvent::EpochStarted,
            ),
            EngineMessage::EpochEnd(ctx) => self.dispatch(
                || EpochCompletedData {
                    index: ctx.index,
                    best: ctx.best.clone(),
                    metrics: ctx.metrics.clone(),
                    score: ctx.score.clone().unwrap_or_default(),
                    objective: ctx.objective.clone(),
                },
                EngineEvent::EpochCompleted,
            ),
            EngineMessage::Improvement(ctx) => self.dispatch(
                || ImprovedData {
                    index: ctx.index,
                    best: ctx.best.clone(),
                    score: ctx.score.clone().unwrap_or_default(),
                },
                EngineEvent::Improved,
            ),
        }
    }

    /// Checks `has_subscribers` for both the specific kind `D` and the
    /// `EngineEvent<T>` wildcard *before* building anything — `build` only
    /// runs (and only clones out of the context) if at least one of the two
    /// is actually true.
    fn dispatch<D, F, W>(&self, build: F, wrap: W)
    where
        D: Send + Sync + 'static,
        F: FnOnce() -> D,
        W: FnOnce(Envelope<D>) -> EngineEvent<T>,
        T: Clone + Send + Sync + 'static,
    {
        let want_specific = self.core.has_subscribers::<Envelope<D>>();
        let want_wildcard = self.core.has_subscribers::<EngineEvent<T>>();
        if !want_specific && !want_wildcard {
            return;
        }

        let payload = Envelope::new(build());
        if want_specific {
            self.core.send(payload.clone());
        }
        if want_wildcard {
            self.core.send(wrap(payload));
        }
    }
}

impl<T> Default for EventBus<T> {
    fn default() -> Self {
        EventBus::new(Arc::new(Executor::default()), ThreadSync::new())
    }
}

impl<T> From<ActorSystem> for EventBus<T> {
    fn from(core: ActorSystem) -> Self {
        EventBus {
            core,
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EvolutionContext;
    use radiate_core::{
        BitChromosome, Ecosystem, EventContext, EventHandler, Front, Message, Objective, Optimize,
        Phenotype, Problem,
    };
    use radiate_test::OneMax;
    use std::sync::Condvar;
    use std::sync::Mutex as StdMutex;
    use std::sync::RwLock;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    // `EvolutionContext` has no `Default` (its `problem` field is a trait
    // object), and its fields are only `pub(crate)` — both fine from here,
    // since this test module is part of the crate. `OneMax` is the same
    // fixture `radiate-test` uses to build real engines elsewhere.
    fn test_context() -> EvolutionContext<BitChromosome, Vec<bool>> {
        let problem: Arc<dyn Problem<BitChromosome, Vec<bool>>> = Arc::new(OneMax::new(4));
        let genotype = problem.encode();
        let best = problem.decode(&genotype);
        let objective = Objective::Single(Optimize::Maximize);

        EvolutionContext {
            ecosystem: Ecosystem::from(vec![Phenotype::from((genotype, 0))]),
            best,
            index: 0,
            metrics: Default::default(),
            score: None,
            front: Arc::new(RwLock::new(Front::new(0..1, objective.clone()))),
            objective,
            problem,
            control: None,
            exprs: None,
            event_system: ActorSystem::default(),
        }
    }

    fn bus_with<M, H>(handler: H) -> EventBus<Vec<bool>>
    where
        M: Message,
        H: EventHandler<M> + 'static,
    {
        let system = ActorSystem::default();
        system.subscribe::<M, _>(handler);
        EventBus::from(system)
    }

    // `EventBus` no longer has `set_sync` (it was dead in production —
    // `build_event_system()` binds the `ThreadSync` on the raw `ActorSystem`
    // before it's ever wrapped), so tests that need a specific `ThreadSync`
    // build the `ActorSystem` with it directly instead.
    fn bus_with_sync<M, H>(sync: ThreadSync, handler: H) -> EventBus<Vec<bool>>
    where
        M: Message,
        H: EventHandler<M> + 'static,
    {
        let system = ActorSystem::with_sync(Arc::new(Executor::default()), sync);
        system.subscribe::<M, _>(handler);
        EventBus::from(system)
    }

    fn wait_for(signal: &Arc<(StdMutex<usize>, Condvar)>, target: usize) {
        let (lock, cv) = &**signal;
        let mut n = lock.lock().unwrap();
        while *n < target {
            let (guard, timeout) = cv.wait_timeout(n, Duration::from_secs(2)).unwrap();
            n = guard;
            if timeout.timed_out() && *n < target {
                panic!("timed out waiting for {target} events, saw {}", *n);
            }
        }
    }

    #[test]
    fn on_start_only_fires_for_start_events() {
        let hits = Arc::new(AtomicUsize::new(0));
        let signal = Arc::new((StdMutex::new(0), Condvar::new()));

        let hits2 = Arc::clone(&hits);
        let signal2 = Arc::clone(&signal);
        let bus = bus_with::<Started, _>(move |_msg: Started, _ctx: &EventContext| {
            hits2.fetch_add(1, Ordering::SeqCst);
            let (n, cv) = &*signal2;
            *n.lock().unwrap() += 1;
            cv.notify_all();
        });

        let ctx = test_context();
        bus.publish(EngineMessage::Start(&ctx));

        wait_for(&signal, 1);
        assert_eq!(hits.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn typed_subscriber_does_not_receive_unrelated_event_kinds() {
        let system = ActorSystem::default();
        system.subscribe::<Started, _>(|_msg: Started, _ctx: &EventContext| {
            panic!("Started handler should not fire for an EpochStart event");
        });

        let epoch_hits = Arc::new(AtomicUsize::new(0));
        let signal = Arc::new((StdMutex::new(0), Condvar::new()));
        let epoch_hits2 = Arc::clone(&epoch_hits);
        let signal2 = Arc::clone(&signal);
        system.subscribe::<EpochStarted, _>(move |_msg: EpochStarted, _ctx: &EventContext| {
            epoch_hits2.fetch_add(1, Ordering::SeqCst);
            let (n, cv) = &*signal2;
            *n.lock().unwrap() += 1;
            cv.notify_all();
        });

        let bus: EventBus<Vec<bool>> = EventBus::from(system);
        let ctx = test_context();
        bus.publish(EngineMessage::EpochStart(&ctx));

        wait_for(&signal, 1);
        assert_eq!(epoch_hits.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn wildcard_subscriber_receives_events_regardless_of_kind() {
        let hits = Arc::new(AtomicUsize::new(0));
        let signal = Arc::new((StdMutex::new(0), Condvar::new()));
        let hits_clone = Arc::clone(&hits);
        let signal_clone = Arc::clone(&signal);
        let bus = bus_with::<EngineEvent<Vec<bool>>, _>(
            move |_event: EngineEvent<Vec<bool>>, _ctx: &EventContext| {
                hits_clone.fetch_add(1, Ordering::SeqCst);
                let (n, cv) = &*signal_clone;
                *n.lock().unwrap() += 1;
                cv.notify_all();
            },
        );

        let ctx = test_context();
        bus.publish(EngineMessage::Start(&ctx));

        wait_for(&signal, 1);
        assert_eq!(hits.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn publish_with_no_subscribers_does_not_panic() {
        let bus: EventBus<Vec<bool>> = EventBus::default();
        let ctx = test_context();
        bus.publish(EngineMessage::Start(&ctx));
    }

    #[test]
    fn handler_can_stop_the_run_via_actor_context() {
        let signal = Arc::new((StdMutex::new(0), Condvar::new()));
        let signal_clone = Arc::clone(&signal);

        let sync = ThreadSync::new();
        let bus = bus_with_sync::<EngineEvent<Vec<bool>>, _>(
            sync.clone(),
            move |_event: EngineEvent<Vec<bool>>, ctx: &EventContext| {
                ctx.stop();
                let (n, cv) = &*signal_clone;
                *n.lock().unwrap() += 1;
                cv.notify_all();
            },
        );
        assert!(!sync.is_stopped());

        let ctx = test_context();
        bus.publish(EngineMessage::Start(&ctx));

        wait_for(&signal, 1);
        assert!(sync.is_stopped());
    }

    #[test]
    fn specific_kind_handler_also_reaches_shared_sync() {
        let signal = Arc::new((StdMutex::new(0), Condvar::new()));
        let signal_clone = Arc::clone(&signal);

        let sync = ThreadSync::new();
        let bus = bus_with_sync::<Improved<Vec<bool>>, _>(
            sync.clone(),
            move |_msg: Improved<Vec<bool>>, ctx: &EventContext| {
                ctx.stop();
                let (n, cv) = &*signal_clone;
                *n.lock().unwrap() += 1;
                cv.notify_all();
            },
        );

        let ctx = test_context();
        bus.publish(EngineMessage::Improvement(&ctx));

        wait_for(&signal, 1);
        assert!(sync.is_stopped());
    }

    #[test]
    fn send_rides_the_same_bus_as_engine_events() {
        #[derive(Clone)]
        struct Warning {
            text: &'static str,
        }

        let seen = Arc::new(StdMutex::new(Vec::new()));
        let signal = Arc::new((StdMutex::new(0), Condvar::new()));

        let seen2 = Arc::clone(&seen);
        let signal2 = Arc::clone(&signal);
        let sync = ThreadSync::new();
        let bus =
            bus_with_sync::<Warning, _>(sync.clone(), move |w: Warning, ctx: &EventContext| {
                seen2.lock().unwrap().push(w.text);
                ctx.stop();
                let (n, cv) = &*signal2;
                *n.lock().unwrap() += 1;
                cv.notify_all();
            });

        bus.send(Warning {
            text: "population diversity collapsing",
        });

        wait_for(&signal, 1);
        assert_eq!(
            *seen.lock().unwrap(),
            vec!["population diversity collapsing"]
        );
        // Same ThreadSync as everything else on this bus, not an
        // independent one — proves it's the same ActorSystem, not a
        // parallel, disconnected one.
        assert!(sync.is_stopped());
    }
}
