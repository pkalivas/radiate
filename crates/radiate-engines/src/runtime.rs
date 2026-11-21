use crate::{
    Engine, EpochComplete, EventHandler, EvolutionContext, Generation, Handler, Limit,
    events::{EngineLogger, Event, GenerationSnapshot, HealthMonitor, LoggingHandler},
};
use crate::{generation::GenerationView, init_logging};
use radiate_core::Expr;
use radiate_core::error::{RadiateResult, Result};
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
                return self.engine.stop();
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
    pub fn context(&self) -> &EvolutionContext<C, T> {
        self.engine.context()
    }

    /// Generic helper function to optionally chain an action based on a condition.
    /// If the condition is true, the provided action function will be executed with the current
    /// runtime instance as its argument. Else, if the condition is false, the runtime instance
    /// will be returned unchanged.
    pub fn chain_if(self, condition: bool, action_fn: impl FnOnce(Self) -> Self) -> Self {
        if condition { action_fn(self) } else { self }
    }

    /// Run the engine. This will begin the execution loop of the engine until it is
    /// stopped by either user or a given limit. There are no checks to see
    /// if limits were provided or not.
    pub fn last(self) -> Result<E::Epoch> {
        self.run()
    }

    /// Executes the provided action function at the specified interval during the engine's execution.
    /// The action is scheduled on the event stream.
    ///
    /// ```rust
    /// use radiate_engines::*;
    ///
    /// let engine = GeneticEngine::builder()
    ///     .codec(FloatCodec::vector(5, 0.0_f32..1.0_f32))
    ///     .fitness_fn(|vals: Vec<f32>| vals.iter().sum::<f32>())
    ///     .build();
    ///
    /// let result = engine
    ///     .iter()
    ///     .take(10)
    ///     // Print the index of every 2nd generation
    ///     .every(2, |view| println!("{:?}", view.index()))
    ///     .last();
    /// ```
    pub fn every<F>(self, interval: usize, mut action_fn: F) -> Self
    where
        F: FnMut(GenerationView<C, T>) + Send + Sync + 'static,
    {
        assert!(interval > 0, "every interval must be greater than zero");
        let guarded_interval = interval.max(1);

        self.engine
            .context()
            .stream()
            .subscribe(move |ctx: &GenerationSnapshot<C, T>| {
                let inner = &ctx.generation;
                action_fn(GenerationView::from(inner.as_ref()));
            })
            .schedule(Expr::every(guarded_interval))
            .unwrap();
        self
    }

    pub fn pause_every(self, interval: usize) -> Self {
        assert!(
            interval > 0,
            "pause_every interval must be greater than zero"
        );
        let guarded_interval = interval.max(1);
        let control = self.engine.context().sync();

        self.engine
            .context()
            .stream()
            .subscribe(move |_: &EpochComplete<T>| {
                control.set_paused(true);
            })
            .schedule(Expr::every(guarded_interval))
            .unwrap();
        self
    }

    /// Executes the provided action function at the specified time interval during the engine's execution.
    /// The action is scheduled on the event stream.
    ///
    /// ```rust
    /// use radiate_engines::*;
    ///
    /// let engine = GeneticEngine::builder()
    ///     .codec(FloatCodec::vector(5, 0.0_f32..1.0_f32))
    ///     .fitness_fn(|vals: Vec<f32>| vals.iter().sum::<f32>())
    ///     .build();
    ///
    /// let result = engine
    ///     .iter()
    ///     .take(10)
    ///     // Print the index every second from the generation that occurs
    ///     // at or right after each second interval
    ///     .throttle(std::time::Duration::from_secs(1), |view| println!("{:?}", view.index()))
    ///     .last();
    /// ```
    pub fn throttle<F>(self, duration: Duration, mut action_fn: F) -> Self
    where
        F: FnMut(GenerationView<C, T>) + Send + Sync + 'static,
    {
        self.engine
            .context()
            .stream()
            .subscribe(move |ctx: &GenerationSnapshot<C, T>| {
                let inner = &ctx.generation;
                action_fn(GenerationView::from(inner.as_ref()));
            })
            .schedule(Expr::throttle(duration))
            .unwrap();
        self
    }

    /// Subscribe to a specific event in the engine's event stream every time it
    /// fires. Certain events are 'lazy', meaning they are only generated and published
    /// when there is an active subscription to that event. Others are only fired when certain
    /// actions are attached to the engine. The event system in the engine acts as a message passing
    /// system, allowing different handlers to react to events as they occur. See [EngineLogger]
    /// for a concrete example of listening/reacting.
    ///
    /// The events that the GeneticEngine emits are:
    ///
    /// * [`EngineStart`](crate::events::EngineStart) - emitted when the engine starts
    /// * [`EngineStop`](crate::events::EngineStop) - emitted when the engine stops
    /// * [`EngineStateChange`](crate::events::EngineStateChange) - emitted when the engine's state changes.
    /// * [`EpochStart`](crate::events::EpochStart) - emitted at the start of each epoch.
    /// * [`Improvement`](crate::events::Improvement) **LAZY** - When the engine finds a new best solution.
    /// * [`EpochComplete`](crate::events::EpochComplete) - emitted at the end of each epoch
    /// * [`CheckpointSaved`](crate::events::CheckpointSaved) - emitted when a checkpoint is saved
    ///     * Only published when Checkpointing is enabled through the [engine's configuration](super::GeneticEngineBuilder::checkpoint).
    /// * [`Warning`](crate::events::Warning) - emitted when a warning occurs through the health monitor
    ///     * Only published when [logging](Self::logging) or [health monitoring](Self::monitor_health) is called on the runtime.
    /// * [`LimitTriggered`](crate::events::LimitTriggered) - emitted when a stopping limit is triggered
    /// * [`GenerationSnapshot`](crate::events::GenerationSnapshot) **LAZY** - emitted for each generation snapshot
    ///     * This is a heavy clone (clones the entire `Ecosystem` and possibly the `Front` if it exists) so use it judiciously.
    /// * [`LogEvent`](crate::events::LogEvent) - emitted for logging purposes.
    ///     * Only emitted when [logging](Self::logging) is enabled. Produces a struct containing
    ///  the appropriate [LogLevel](crate::events::LogLevel) and a message of what occurred. Logs can contain
    ///  information about limits being triggered, checkpoints being saved, epochs completing, state changes,
    ///  and other significant events within the engine's lifecycle. This is great for collecting simple logs without
    ///  having to manually subscribe to each individual event.
    ///
    /// ## Simple fn subscriber
    /// ```rust
    /// use radiate_engines::*;
    ///
    /// let engine = GeneticEngine::builder()
    ///     .codec(FloatCodec::vector(5, 0.0_f32..1.0_f32))
    ///     .fitness_fn(|vals: Vec<f32>| vals.iter().sum::<f32>())
    ///     .build();
    ///
    /// let result = engine
    ///     .iter()
    ///     .take(10)
    ///     // Subscribe to the start of each epoch and print the event
    ///     .subscribe::<EpochStart>(|event: &EpochStart| println!("{:?}", event))
    ///     .last();
    /// ```
    ///
    /// ## Concrete subscriber
    ///
    /// Its also possible to use a concrete handler so you can listen and possibly react/publish
    /// other events as well.
    ///
    /// ```rust
    /// use radiate_engines::*;
    ///
    /// struct MyHandler;
    ///
    /// impl Handler<EpochStart> for MyHandler {
    ///     fn handle(&mut self, event: &EpochStart, ctx: &EventContext<'_, Self>) {
    ///         println!("Received epoch start: {:?}", event);
    ///         // Publish a log event indicating the epoch has started
    ///         ctx.publish(LogEvent(
    ///             LogLevel::Info,
    ///             format!("Epoch started: {:?}", event),
    ///         ));
    ///     }
    /// }
    ///
    /// let engine = GeneticEngine::builder()
    ///     .codec(FloatCodec::vector(5, 0.0_f32..1.0_f32))
    ///     .fitness_fn(|vals: Vec<f32>| vals.iter().sum::<f32>())
    ///     .build();
    ///
    /// let result = engine
    ///     .iter()
    ///     .take(10)
    ///     // Subscribe using a concrete handler
    ///     .subscribe(MyHandler)
    ///     .last();
    /// ```
    pub fn subscribe<EV: Event>(self, handler: impl Handler<EV>) -> Self {
        self.engine.context().stream().subscribe(handler);
        self
    }

    pub fn attach<H: EventHandler>(self, handler: H) -> Self {
        self.engine.context().stream().attach(handler).unwrap();
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

        let stream = self.engine.context().stream();
        stream.detach::<LoggingHandler>();

        if !stream.handler_registered::<LoggingHandler>() {
            stream.attach(LoggingHandler::new()).unwrap();
        }

        if !stream.handler_registered::<HealthMonitor<T>>() {
            stream.attach(HealthMonitor::<T>::default()).unwrap();
        }

        if !stream.handler_registered::<EngineLogger<T>>() {
            stream.attach(EngineLogger::<T>::new()).unwrap();
        }

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
