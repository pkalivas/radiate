use crate::context::EvolutionContext;
use radiate_core::{ActorSystem, Chromosome, Envelope, MetricSet, Objective, Score};
use std::fmt::Debug;

/// Checks `has_subscribers` for both the specific kind `D` and the
/// `EngineEvent<T>` wildcard *before* building anything — `build` only runs
/// (and only clones out of the context) if at least one of the two is
/// actually true. Shared by the five `dispatch_*` functions below, each of
/// which just supplies its own `build`/`wrap` for one event kind.
fn dispatch_kind<T, D, F, W>(system: &ActorSystem, build: F, wrap: W)
where
    T: Clone + Send + Sync + 'static,
    D: Send + Sync + 'static,
    F: FnOnce() -> D,
    W: FnOnce(Envelope<D>) -> EngineEvent<T>,
{
    let want_specific = system.has_subscribers::<Envelope<D>>();
    let want_wildcard = system.has_subscribers::<EngineEvent<T>>();
    if !want_specific && !want_wildcard {
        return;
    }

    let payload = Envelope::new(build());
    if want_specific {
        system.send(payload.clone());
    }
    if want_wildcard {
        system.send(wrap(payload));
    }
}

/// Announces the run has started. Dispatched once, the first time
/// `GeneticEngine::step()` runs.
pub(crate) fn dispatch_start<C, T>(_ctx: &EvolutionContext<C, T>, system: &ActorSystem)
where
    C: Chromosome,
    T: Clone + Send + Sync + 'static,
{
    dispatch_kind::<T, _, _, _>(system, || StartedData, EngineEvent::Started)
}

/// Announces the run has stopped. Dispatched from `GeneticEngine`'s `Drop`.
pub(crate) fn dispatch_stop<C, T>(ctx: &EvolutionContext<C, T>, system: &ActorSystem)
where
    C: Chromosome,
    T: Clone + Send + Sync + 'static,
{
    dispatch_kind::<T, _, _, _>(
        system,
        || StoppedData {
            index: ctx.index,
            best: ctx.best.clone(),
            metrics: ctx.metrics.clone(),
            score: ctx.score.clone().unwrap_or_default(),
        },
        EngineEvent::Stopped,
    )
}

/// Announces a generation is about to run.
pub(crate) fn dispatch_epoch_start<C, T>(ctx: &EvolutionContext<C, T>, system: &ActorSystem)
where
    C: Chromosome,
    T: Clone + Send + Sync + 'static,
{
    dispatch_kind::<T, _, _, _>(
        system,
        || EpochStartedData { index: ctx.index },
        EngineEvent::EpochStarted,
    )
}

/// Announces a generation has finished.
pub(crate) fn dispatch_epoch_end<C, T>(ctx: &EvolutionContext<C, T>, system: &ActorSystem)
where
    C: Chromosome,
    T: Clone + Send + Sync + 'static,
{
    dispatch_kind::<T, _, _, _>(
        system,
        || EpochCompletedData {
            index: ctx.index,
            best: ctx.best.clone(),
            metrics: ctx.metrics.clone(),
            score: ctx.score.clone().unwrap_or_default(),
            objective: ctx.objective.clone(),
        },
        EngineEvent::EpochCompleted,
    )
}

/// Announces the best individual improved this generation.
pub(crate) fn dispatch_improvement<C, T>(ctx: &EvolutionContext<C, T>, system: &ActorSystem)
where
    C: Chromosome,
    T: Clone + Send + Sync + 'static,
{
    dispatch_kind::<T, _, _, _>(
        system,
        || ImprovedData {
            index: ctx.index,
            best: ctx.best.clone(),
            score: ctx.score.clone().unwrap_or_default(),
        },
        EngineEvent::Improved,
    )
}

#[derive(Clone, Debug)]
pub struct LimitTriggered {
    pub generation: usize,
    pub kind: &'static str,
    pub description: String,
}

pub struct StartedData;
pub type Started = Envelope<StartedData>;

pub struct StoppedData<T> {
    pub index: usize,
    pub best: T,
    pub metrics: MetricSet,
    pub score: Score,
}
pub type Stopped<T> = Envelope<StoppedData<T>>;

pub struct EpochStartedData {
    pub index: usize,
}
pub type EpochStarted = Envelope<EpochStartedData>;

pub struct EpochCompletedData<T> {
    pub index: usize,
    pub best: T,
    pub metrics: MetricSet,
    pub score: Score,
    pub objective: Objective,
}
pub type EpochCompleted<T> = Envelope<EpochCompletedData<T>>;

pub struct ImprovedData<T> {
    pub index: usize,
    pub best: T,
    pub score: Score,
}
pub type Improved<T> = Envelope<ImprovedData<T>>;

/// The "give me every kind of engine event" umbrella — one type, so a single
/// wildcard subscription still works, at the cost of a second `send` (gated
/// by its own `has_subscribers` check, same as the specific-kind path) when
/// both a wildcard and a specific-kind subscriber exist for the same event.
#[derive(Clone)]
pub enum EngineEvent<T> {
    Started(Started),
    Stopped(Stopped<T>),
    EpochStarted(EpochStarted),
    EpochCompleted(EpochCompleted<T>),
    Improved(Improved<T>),
}

impl<T> EngineEvent<T> {
    pub fn is_start(&self) -> bool {
        matches!(self, EngineEvent::Started(_))
    }

    pub fn is_stop(&self) -> bool {
        matches!(self, EngineEvent::Stopped(_))
    }

    pub fn is_epoch_start(&self) -> bool {
        matches!(self, EngineEvent::EpochStarted(_))
    }

    pub fn is_epoch_complete(&self) -> bool {
        matches!(self, EngineEvent::EpochCompleted(_))
    }

    pub fn is_improvement(&self) -> bool {
        matches!(self, EngineEvent::Improved(_))
    }
}

impl<T: Debug> Debug for EngineEvent<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineEvent::Started(_) => write!(f, "Started"),
            EngineEvent::Stopped(s) => write!(
                f,
                "Stopped(index={}, best={:?}, score={:?})",
                s.index, s.best, s.score
            ),
            EngineEvent::EpochStarted(s) => write!(f, "EpochStarted(index={})", s.index),
            EngineEvent::EpochCompleted(s) => write!(
                f,
                "EpochCompleted(index={}, best={:?}, score={:?}, objective={:?})",
                s.index, s.best, s.score, s.objective
            ),
            EngineEvent::Improved(s) => write!(
                f,
                "Improved(index={}, best={:?}, score={:?})",
                s.index, s.best, s.score
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EvolutionContext;
    use radiate_core::{
        BitChromosome, Ecosystem, EventContext, EventHandler, Executor, Front, Message, Objective,
        Optimize, Phenotype, Problem, ThreadSync,
    };
    use radiate_test::OneMax;
    use std::sync::Arc;
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

    fn system_with<M, H>(handler: H) -> ActorSystem
    where
        M: Message,
        H: EventHandler<M> + 'static,
    {
        let system = ActorSystem::default();
        system.subscribe::<M, _>(handler);
        system
    }

    fn system_with_sync<M, H>(sync: ThreadSync, handler: H) -> ActorSystem
    where
        M: Message,
        H: EventHandler<M> + 'static,
    {
        let system = ActorSystem::with_sync(Arc::new(Executor::default()), sync);
        system.subscribe::<M, _>(handler);
        system
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
        let system = system_with::<Started, _>(move |_msg: Started, _ctx: &EventContext| {
            hits2.fetch_add(1, Ordering::SeqCst);
            let (n, cv) = &*signal2;
            *n.lock().unwrap() += 1;
            cv.notify_all();
        });

        let ctx = test_context();
        dispatch_start(&ctx, &system);

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

        let ctx = test_context();
        dispatch_epoch_start(&ctx, &system);

        wait_for(&signal, 1);
        assert_eq!(epoch_hits.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn wildcard_subscriber_receives_events_regardless_of_kind() {
        let hits = Arc::new(AtomicUsize::new(0));
        let signal = Arc::new((StdMutex::new(0), Condvar::new()));
        let hits_clone = Arc::clone(&hits);
        let signal_clone = Arc::clone(&signal);
        let system = system_with::<EngineEvent<Vec<bool>>, _>(
            move |_event: EngineEvent<Vec<bool>>, _ctx: &EventContext| {
                hits_clone.fetch_add(1, Ordering::SeqCst);
                let (n, cv) = &*signal_clone;
                *n.lock().unwrap() += 1;
                cv.notify_all();
            },
        );

        let ctx = test_context();
        dispatch_start(&ctx, &system);

        wait_for(&signal, 1);
        assert_eq!(hits.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn dispatch_with_no_subscribers_does_not_panic() {
        let system = ActorSystem::default();
        let ctx = test_context();
        dispatch_start(&ctx, &system);
    }

    #[test]
    fn handler_can_stop_the_run_via_actor_context() {
        let signal = Arc::new((StdMutex::new(0), Condvar::new()));
        let signal_clone = Arc::clone(&signal);

        let sync = ThreadSync::new();
        let system = system_with_sync::<EngineEvent<Vec<bool>>, _>(
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
        dispatch_start(&ctx, &system);

        wait_for(&signal, 1);
        assert!(sync.is_stopped());
    }

    #[test]
    fn specific_kind_handler_also_reaches_shared_sync() {
        let signal = Arc::new((StdMutex::new(0), Condvar::new()));
        let signal_clone = Arc::clone(&signal);

        let sync = ThreadSync::new();
        let system = system_with_sync::<Improved<Vec<bool>>, _>(
            sync.clone(),
            move |_msg: Improved<Vec<bool>>, ctx: &EventContext| {
                ctx.stop();
                let (n, cv) = &*signal_clone;
                *n.lock().unwrap() += 1;
                cv.notify_all();
            },
        );

        let ctx = test_context();
        dispatch_improvement(&ctx, &system);

        wait_for(&signal, 1);
        assert!(sync.is_stopped());
    }

    #[test]
    fn send_rides_the_same_system_as_engine_events() {
        #[derive(Clone)]
        struct Warning {
            text: &'static str,
        }

        let seen = Arc::new(StdMutex::new(Vec::new()));
        let signal = Arc::new((StdMutex::new(0), Condvar::new()));

        let seen2 = Arc::clone(&seen);
        let signal2 = Arc::clone(&signal);
        let sync = ThreadSync::new();
        let system =
            system_with_sync::<Warning, _>(sync.clone(), move |w: Warning, ctx: &EventContext| {
                seen2.lock().unwrap().push(w.text);
                ctx.stop();
                let (n, cv) = &*signal2;
                *n.lock().unwrap() += 1;
                cv.notify_all();
            });

        system.send(Warning {
            text: "population diversity collapsing",
        });

        wait_for(&signal, 1);
        assert_eq!(
            *seen.lock().unwrap(),
            vec!["population diversity collapsing"]
        );
        // Same ThreadSync as everything else on this system, not an
        // independent one — proves it's the same ActorSystem, not a
        // parallel, disconnected one.
        assert!(sync.is_stopped());
    }
}
