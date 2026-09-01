use crate::{
    Engine, EventHandler, EvolutionContext, Generation, Limit,
    events::{EngineLogger, Event, GenerationSnapshot, HealthMonitor, LoggingHandler},
};
use crate::{generation::GenerationView, init_logging};
use radiate_core::error::{RadiateResult, Result};
use radiate_core::rate::Expr;
use radiate_core::{Chromosome, EngineState, Score};
use std::collections::VecDeque;
use std::time::Duration;

pub trait RuntimeLimit<E: Engine> {
    fn proceed(&mut self, context: &E::Ctx) -> RadiateResult<bool>;
}

pub struct EngineRuntime<E: Engine> {
    engine: E,
    limits: Vec<Box<dyn RuntimeLimit<E>>>,
}

impl<E: Engine> EngineRuntime<E> {
    pub fn new(engine: E) -> Self {
        Self {
            engine,
            limits: Vec::new(),
        }
    }

    #[inline]
    pub fn run(mut self) -> Result<E::Epoch> {
        loop {
            if matches!(self.engine.state(), EngineState::Stopped) {
                return Ok(self.engine.epoch());
            }

            self.step()?;
        }
    }

    #[inline]
    fn step(&mut self) -> Result<()> {
        if matches!(self.engine.state(), EngineState::Stopped) {
            return Ok(());
        }

        self.engine.step()?;

        let ctx = self.engine.context();

        for limit in self.limits.iter_mut() {
            if !limit.proceed(ctx)? {
                self.engine.stop();
                return Ok(());
            }
        }

        Ok(())
    }

    fn add_limit<L>(&mut self, limit: L)
    where
        L: RuntimeLimit<E> + 'static,
    {
        let boxed: Box<dyn RuntimeLimit<E>> = Box::new(limit);
        self.limits.push(boxed);
    }
}

/// General iter fns for the `EngineRuntime` struct, allowing for a more ergonomic
/// and fluent interface when configuring the runtime.
impl<C, T, E> EngineRuntime<E>
where
    E: Engine<Epoch = Generation<C, T>, Ctx = EvolutionContext<C, T>>,
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    pub fn chain_if(self, condition: bool, action_fn: impl FnOnce(Self) -> Self) -> Self {
        if condition { action_fn(self) } else { self }
    }

    pub fn last(self) -> Result<E::Epoch> {
        self.run()
    }

    pub fn every<F>(self, interval: usize, mut action_fn: F) -> Self
    where
        F: FnMut(GenerationView<C, T>) + Send + Sync + 'static,
    {
        assert!(interval > 0, "every interval must be greater than zero");
        let guarded_interval = interval.max(1);

        self.engine
            .context()
            .events()
            .subscribe(move |ctx: &GenerationSnapshot<C, T>| {
                let inner = &ctx.generation;
                if inner.index().is_multiple_of(guarded_interval) {
                    action_fn(GenerationView::from(inner.as_ref()));
                }
            })
            .schedule(guarded_interval);
        self
    }

    pub fn on<EV: Event>(self, handler: impl EventHandler<EV>) -> Self {
        self.engine.context().events().subscribe(handler);
        self
    }
}

/// Limit configuration methods for the `EngineRuntime` struct, allowing users to specify various
/// stopping conditions for the evolutionary process.
impl<C, T, E> EngineRuntime<E>
where
    E: Engine<Epoch = Generation<C, T>, Ctx = EvolutionContext<C, T>>,
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    pub fn until_score(mut self, score: impl Into<Score>) -> EngineRuntime<E> {
        self.add_limit(Limit::Score(score.into()));
        self
    }

    pub fn until_generation(mut self, generations: usize) -> EngineRuntime<E> {
        self.add_limit(Limit::Generation(generations));
        self
    }

    pub fn until_seconds(mut self, seconds: f64) -> EngineRuntime<E> {
        self.add_limit(Limit::Seconds(Duration::from_secs_f64(seconds)));
        self
    }

    pub fn until_duration(mut self, duration: impl Into<std::time::Duration>) -> EngineRuntime<E> {
        self.add_limit(Limit::Seconds(duration.into()));
        self
    }

    pub fn until_convergence(mut self, window: usize, epsilon: f32) -> EngineRuntime<E> {
        self.add_limit(Limit::Convergence(
            window,
            epsilon,
            VecDeque::with_capacity(window),
        ));
        self
    }

    pub fn until_expr(mut self, expr: impl Into<Expr>) -> EngineRuntime<E> {
        self.add_limit(Limit::Expr(expr.into()));
        self
    }

    pub fn until<F>(mut self, limit: F) -> EngineRuntime<E>
    where
        C: 'static,
        F: Fn(GenerationView<C, T>) -> bool + 'static,
    {
        self.add_limit(limit);
        self
    }

    pub fn limit(self, limit: impl Into<Limit>) -> EngineRuntime<E> {
        let limit = limit.into();
        match limit {
            Limit::Generation(gens) => self.until_generation(gens),
            Limit::Seconds(secs) => self.until_duration(secs),
            Limit::Score(score) => self.until_score(score),
            Limit::Convergence(window, epsilon, _) => self.until_convergence(window, epsilon),
            Limit::Expr(expr) => self.until_expr(expr),
            Limit::Combined(lims) => lims
                .into_iter()
                .fold(self, |runtime, limit| runtime.limit(limit)),
            Limit::Fn => self,
        }
    }

    pub fn take(self, count: usize) -> EngineRuntime<E> {
        self.until_generation(count)
    }

    pub fn take_while<F>(self, predicate: F) -> EngineRuntime<E>
    where
        C: 'static,
        F: Fn(GenerationView<C, T>) -> bool + 'static,
    {
        self.until(move |view: GenerationView<C, T>| -> bool { !predicate(view) })
    }
}

/// Action based configuration methods for the `EngineRuntime` struct, allowing users to specify various
/// actions to be executed during the evolutionary process.
impl<C, T, E> EngineRuntime<E>
where
    E: Engine<Epoch = Generation<C, T>, Ctx = EvolutionContext<C, T>>,
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    pub fn logging(self) -> EngineRuntime<E> {
        init_logging();
        let stream = self.engine.context().events();

        stream.register(EngineLogger::<T>::new());
        stream.register(HealthMonitor::<T>::default());
        stream.subscribe(LoggingHandler);

        self
    }
}

impl<E> Iterator for EngineRuntime<E>
where
    E: Engine + 'static,
{
    type Item = E::Epoch;

    fn next(&mut self) -> Option<Self::Item> {
        if matches!(self.engine.state(), EngineState::Stopped) {
            return None;
        }

        self.step().ok()?;
        Some(self.engine.epoch())
    }
}
