use crate::context::RuntimeContext;
use crate::generation::GenerationView;
use crate::runtime::RuntimeAction;
use crate::runtime::actions::LoggingAction;
use crate::runtime::condition::{
    ConvergenceLimit, DurationLimit, ExprLimit, GenerationLimit, MetricLimit, RuntimeLimit,
    ScoreLimit,
};
use crate::{Engine, EngineControl, EvolutionContext, Generation, Limit, init_logging};
#[cfg(feature = "serde")]
use crate::{FileWriter, JsonWriter};
use radiate_core::error::Result;
use radiate_core::{Chromosome, Expr, Metric, Score, radiate_err};
#[cfg(feature = "serde")]
use serde::Serialize;
#[cfg(feature = "serde")]
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

pub(crate) struct EngineGuard<'a, E: Engine> {
    engine: &'a E,
}

impl<'a, E: Engine> EngineGuard<'a, E> {
    pub fn new(engine: &'a E) -> Self {
        EngineGuard { engine }
    }

    pub fn view(&self) -> &E::Context {
        self.engine.context()
    }

    pub fn epoch(&self) -> E::Epoch {
        self.engine.epoch()
    }
}

pub struct EngineRuntime<E>
where
    E: Engine,
    E::Context: RuntimeContext,
{
    engine: E,
    control: Option<EngineControl>,
    actions: Option<Vec<Box<dyn RuntimeAction<E>>>>,
    limits: Option<Vec<Box<dyn RuntimeLimit<E>>>>,
    done: bool,
}

impl<E> EngineRuntime<E>
where
    E: Engine,
    E::Context: RuntimeContext,
{
    pub fn new(engine: E, control: Option<EngineControl>) -> Self {
        Self {
            engine,
            control,
            actions: None,
            limits: None,
            done: false,
        }
    }

    #[inline]
    pub fn run(mut self) -> Result<E::Epoch> {
        loop {
            if self.done {
                return Ok(self.engine.epoch());
            }

            self.step()?;
        }
    }

    #[inline]
    fn step(&mut self) -> Result<()> {
        if self.done {
            return Err(radiate_err!("Engine has already completed"));
        }

        if let Some(control) = &self.control
            && control.is_stopped()
        {
            self.done = true;
            return Ok(());
        }

        self.engine.step()?;
        let guard = EngineGuard::new(&self.engine);

        if let Some(actions) = &mut self.actions {
            for action in actions.iter_mut() {
                action.execute(&guard);
            }
        }

        if let Some(limits) = &mut self.limits {
            for limit in limits.iter_mut() {
                if !limit.check(&guard) {
                    self.done = true;
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    pub fn until_score(mut self, score: impl Into<Score>) -> EngineRuntime<E> {
        let limit = ScoreLimit::from(score);
        self.add_limit(limit);
        self
    }

    pub fn until_generation(mut self, generations: usize) -> EngineRuntime<E> {
        let limit = GenerationLimit::from(generations);
        self.add_limit(limit);
        self
    }

    pub fn until_seconds(mut self, seconds: f64) -> EngineRuntime<E> {
        let limit = DurationLimit::from(Duration::from_secs_f64(seconds));
        self.add_limit(limit);
        self
    }

    pub fn until_duration(mut self, duration: impl Into<std::time::Duration>) -> EngineRuntime<E> {
        let limit = DurationLimit::from(duration);
        self.add_limit(limit);
        self
    }

    pub fn until_convergence(mut self, window: usize, epsilon: f32) -> EngineRuntime<E> {
        let limit = ConvergenceLimit::new(window, epsilon);
        self.add_limit(limit);
        self
    }

    pub fn until_expr(mut self, expr: impl Into<Expr>) -> EngineRuntime<E> {
        let limit = ExprLimit::from(expr.into());
        self.add_limit(limit);
        self
    }

    pub fn until_metric(
        mut self,
        name: &str,
        predicate: Arc<dyn Fn(&Metric) -> bool + Send + Sync>,
    ) -> EngineRuntime<E> {
        let limit = MetricLimit::new(name.into(), predicate);
        self.add_limit(limit);
        self
    }

    pub fn limit(self, limit: impl Into<Limit>) -> EngineRuntime<E> {
        let limit = limit.into();
        match limit {
            Limit::Generation(gens) => self.until_generation(gens),
            Limit::Seconds(secs) => self.until_duration(secs),
            Limit::Score(score) => self.until_score(score),
            Limit::Convergence(window, epsilon) => self.until_convergence(window, epsilon),
            Limit::Expr(expr) => self.until_expr(expr),
            Limit::Metric(name, predicate) => self.until_metric(&name, predicate),
            Limit::Combined(lims) => lims
                .into_iter()
                .fold(self, |runtime, limit| runtime.limit(limit)),
        }
    }

    pub fn take(self, count: usize) -> EngineRuntime<E> {
        self.until_generation(count)
    }

    pub fn last(self) -> Result<E::Epoch> {
        self.run()
    }

    pub fn logging(self) -> EngineRuntime<E> {
        self.log_every(1)
    }

    pub fn log_every(mut self, every: usize) -> EngineRuntime<E> {
        init_logging();
        let action = LoggingAction(every);
        self.add_action(action);
        self
    }

    pub fn chain_if(self, condition: bool, action_fn: impl FnOnce(Self) -> Self) -> Self {
        if condition { action_fn(self) } else { self }
    }

    fn add_limit<L>(&mut self, limit: L)
    where
        L: RuntimeLimit<E> + 'static,
    {
        let boxed: Box<dyn RuntimeLimit<E>> = Box::new(limit);
        if let Some(limits) = &mut self.limits {
            limits.push(boxed);
        } else {
            self.limits = Some(vec![boxed]);
        }
    }

    fn add_action<A>(&mut self, action: A)
    where
        A: RuntimeAction<E> + 'static,
    {
        let boxed: Box<dyn RuntimeAction<E>> = Box::new(action);
        if let Some(actions) = &mut self.actions {
            actions.push(boxed);
        } else {
            self.actions = Some(vec![boxed]);
        }
    }
}

#[cfg(feature = "serde")]
impl<E> EngineRuntime<E>
where
    E: Engine + 'static,
    E::Context: RuntimeContext,
    E::Epoch: Serialize,
{
    pub fn checkpoint(mut self, interval: usize, folder_path: impl AsRef<Path>) -> EngineRuntime<E>
    where
        E: Engine + 'static,
        E::Epoch: Serialize,
    {
        use crate::runtime::actions::CheckpointAction;

        let path_without_extension = folder_path
            .as_ref()
            .to_str()
            .and_then(|s| s.rsplit('.').nth(1))
            .unwrap_or(folder_path.as_ref().to_str().unwrap_or("checkpoints"));

        let action = CheckpointAction {
            interval,
            path: PathBuf::from(path_without_extension),
            writer: Box::new(JsonWriter),
        };

        self.add_action(action);

        self
    }

    pub fn checkpoint_with(
        mut self,
        interval: usize,
        folder_path: impl AsRef<Path>,
        writer: Box<dyn FileWriter<E::Epoch>>,
    ) -> EngineRuntime<E>
    where
        E: Engine + 'static,
        E::Epoch: Serialize,
    {
        use crate::runtime::actions::CheckpointAction;

        let path_without_extension = folder_path
            .as_ref()
            .to_str()
            .and_then(|s| s.rsplit('.').nth(1))
            .unwrap_or(folder_path.as_ref().to_str().unwrap_or("checkpoints"));

        let action = CheckpointAction {
            interval,
            path: PathBuf::from(path_without_extension),
            writer,
        };

        self.add_action(action);
        self
    }
}

impl<C, T, E> EngineRuntime<E>
where
    E: Engine<Epoch = Generation<C, T>, Context = EvolutionContext<C, T>> + 'static,
    E::Context: RuntimeContext,
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    pub fn until<F>(mut self, limit: F) -> EngineRuntime<E>
    where
        E: 'static,
        F: Fn(GenerationView<C, T>) -> bool + 'static,
    {
        pub(crate) struct FnLimit<Ch, V, F>
        where
            F: Fn(GenerationView<Ch, V>) -> bool,
        {
            condition: F,
            _chrome: std::marker::PhantomData<Ch>,
            _value: std::marker::PhantomData<V>,
        }

        impl<Ch, V, F> FnLimit<Ch, V, F>
        where
            F: Fn(GenerationView<Ch, V>) -> bool,
        {
            pub fn new(condition: F) -> Self {
                FnLimit {
                    condition,
                    _chrome: std::marker::PhantomData,
                    _value: std::marker::PhantomData,
                }
            }
        }

        impl<Ch, V, F, E> RuntimeLimit<E> for FnLimit<Ch, V, F>
        where
            E: Engine<Epoch = Generation<Ch, V>, Context = EvolutionContext<Ch, V>>,
            Ch: Chromosome,
            F: Fn(GenerationView<Ch, V>) -> bool,
        {
            fn check<'a>(&mut self, snapshot: &EngineGuard<'a, E>) -> bool {
                let view = GenerationView::new(snapshot.view());
                !(self.condition)(view)
            }
        }

        let limit = FnLimit::new(limit);
        self.add_limit(limit);
        self
    }
}

impl<E> Iterator for EngineRuntime<E>
where
    E: Engine + 'static,
    E::Context: RuntimeContext,
{
    type Item = E::Epoch;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        self.step().ok()?;
        Some(self.engine.epoch())
    }
}
