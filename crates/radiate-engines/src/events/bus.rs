use crate::events::message::*;
use radiate_core::{ActorSystem, Chromosome, Envelope, EventHandler, Executor, ThreadSync};
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

    /// Rebind the executor used for future dispatch without disturbing any
    /// subscriber already registered — used by the builder, which collects
    /// subscribers before the real executor is known.
    pub fn set_executor(&self, executor: Arc<Executor>) {
        self.core.set_executor(executor);
    }

    /// Rebind the `ThreadSync` handed to actors from this point on — used by
    /// the builder to bind this bus to the same control primitive the
    /// engine's `EvolutionContext` ends up using, so `ctx.sync` in any
    /// handler and `engine.control()` are the same object.
    pub fn set_sync(&self, sync: ThreadSync) {
        self.core.set_sync(sync);
    }

    pub fn subscribe<H>(&self, handler: H)
    where
        H: EventHandler<EngineEvent<T>> + 'static,
        T: Send + Sync + 'static,
    {
        self.core.subscribe::<EngineEvent<T>, _>(handler);
    }

    pub fn on_start<H>(&self, handler: H)
    where
        H: EventHandler<Started> + 'static,
    {
        self.core.subscribe::<Started, _>(handler);
    }

    pub fn on_stop<H>(&self, handler: H)
    where
        H: EventHandler<Stopped<T>> + 'static,
        T: Send + Sync + 'static,
    {
        self.core.subscribe::<Stopped<T>, _>(handler);
    }

    pub fn on_epoch_start<H>(&self, handler: H)
    where
        H: EventHandler<EpochStarted> + 'static,
    {
        self.core.subscribe::<EpochStarted, _>(handler);
    }

    pub fn on_epoch_complete<H>(&self, handler: H)
    where
        H: EventHandler<EpochCompleted<T>> + 'static,
        T: Send + Sync + 'static,
    {
        self.core.subscribe::<EpochCompleted<T>, _>(handler);
    }

    pub fn on_improvement<H>(&self, handler: H)
    where
        H: EventHandler<Improved<T>> + 'static,
        T: Send + Sync + 'static,
    {
        self.core.subscribe::<Improved<T>, _>(handler);
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
}

impl<T> Default for EventBus<T> {
    fn default() -> Self {
        EventBus::new(Arc::new(Executor::default()), ThreadSync::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EvolutionContext;
    use radiate_core::{
        ActorContext, BitChromosome, Ecosystem, Front, Objective, Optimize, Phenotype, Problem,
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
        }
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
        let bus: EventBus<Vec<bool>> = EventBus::default();
        let hits = Arc::new(AtomicUsize::new(0));
        let signal = Arc::new((StdMutex::new(0), Condvar::new()));

        let hits2 = Arc::clone(&hits);
        let signal2 = Arc::clone(&signal);
        bus.on_start(move |_msg: Started, _ctx: &ActorContext| {
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
        let bus: EventBus<Vec<bool>> = EventBus::default();

        bus.on_start(|_msg: Started, _ctx: &ActorContext| {
            panic!("on_start handler should not fire for an EpochStart event");
        });

        let epoch_hits = Arc::new(AtomicUsize::new(0));
        let signal = Arc::new((StdMutex::new(0), Condvar::new()));
        let epoch_hits2 = Arc::clone(&epoch_hits);
        let signal2 = Arc::clone(&signal);
        bus.on_epoch_start(move |_msg: EpochStarted, _ctx: &ActorContext| {
            epoch_hits2.fetch_add(1, Ordering::SeqCst);
            let (n, cv) = &*signal2;
            *n.lock().unwrap() += 1;
            cv.notify_all();
        });

        let ctx = test_context();
        bus.publish(EngineMessage::EpochStart(&ctx));

        wait_for(&signal, 1);
        assert_eq!(epoch_hits.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn wildcard_subscriber_receives_events_regardless_of_kind() {
        let bus: EventBus<Vec<bool>> = EventBus::default();

        let hits = Arc::new(AtomicUsize::new(0));
        let signal = Arc::new((StdMutex::new(0), Condvar::new()));
        let hits_clone = Arc::clone(&hits);
        let signal_clone = Arc::clone(&signal);
        bus.subscribe(move |_event: EngineEvent<Vec<bool>>, _ctx: &ActorContext| {
            hits_clone.fetch_add(1, Ordering::SeqCst);
            let (n, cv) = &*signal_clone;
            *n.lock().unwrap() += 1;
            cv.notify_all();
        });

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
        let bus: EventBus<Vec<bool>> = EventBus::default();
        let signal = Arc::new((StdMutex::new(0), Condvar::new()));
        let signal_clone = Arc::clone(&signal);

        bus.subscribe(move |_event: EngineEvent<Vec<bool>>, ctx: &ActorContext| {
            ctx.sync.stop();
            let (n, cv) = &*signal_clone;
            *n.lock().unwrap() += 1;
            cv.notify_all();
        });

        let sync = ThreadSync::new();
        bus.set_sync(sync.clone());
        assert!(!sync.is_stopped());

        let ctx = test_context();
        bus.publish(EngineMessage::Start(&ctx));

        wait_for(&signal, 1);
        assert!(sync.is_stopped());
    }

    #[test]
    fn specific_kind_handler_also_reaches_shared_sync() {
        let bus: EventBus<Vec<bool>> = EventBus::default();
        let signal = Arc::new((StdMutex::new(0), Condvar::new()));
        let signal_clone = Arc::clone(&signal);

        bus.on_improvement(move |_msg: Improved<Vec<bool>>, ctx: &ActorContext| {
            ctx.sync.stop();
            let (n, cv) = &*signal_clone;
            *n.lock().unwrap() += 1;
            cv.notify_all();
        });

        let sync = ThreadSync::new();
        bus.set_sync(sync.clone());

        let ctx = test_context();
        bus.publish(EngineMessage::Improvement(&ctx));

        wait_for(&signal, 1);
        assert!(sync.is_stopped());
    }
}
