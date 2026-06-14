use crate::actions::LoggingAction;
use crate::generation::GenerationView;
use crate::{Engine, EngineControl, EvolutionContext, Generation, Limit, init_logging};
#[cfg(feature = "serde")]
use crate::{FileWriter, JsonWriter};
use radiate_core::error::{RadiateResult, Result};
use radiate_core::{Chromosome, Expr, Metric, Score, radiate_err};
#[cfg(feature = "serde")]
use serde::Serialize;
use std::collections::VecDeque;
#[cfg(feature = "serde")]
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

pub trait RuntimeLimit<E: Engine> {
    fn proceed(&mut self, snapshot: &E::Ctx) -> RadiateResult<bool>;
}

pub trait RuntimeAction<E: Engine> {
    fn execute(&mut self, guard: &E::Ctx) -> RadiateResult<()>;
}

pub struct EngineRuntime<E: Engine> {
    engine: E,
    control: Option<EngineControl>,
    actions: Option<Vec<Box<dyn RuntimeAction<E>>>>,
    limits: Option<Vec<Box<dyn RuntimeLimit<E>>>>,
    done: bool,
}

impl<E: Engine> EngineRuntime<E> {
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

        if let Some(actions) = &mut self.actions {
            let ctx = self.engine.context();
            for action in actions.iter_mut() {
                action.execute(ctx)?;
            }
        }

        if let Some(limits) = &mut self.limits {
            let ctx = self.engine.context();
            for limit in limits.iter_mut() {
                if !limit.proceed(ctx)? {
                    self.done = true;
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    pub fn chain_if(self, condition: bool, action_fn: impl FnOnce(Self) -> Self) -> Self {
        if condition { action_fn(self) } else { self }
    }

    pub fn last(self) -> Result<E::Epoch> {
        self.run()
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

    pub fn until_metric(
        mut self,
        name: &str,
        predicate: Arc<dyn Fn(&Metric) -> bool + Send + Sync>,
    ) -> EngineRuntime<E> {
        self.add_limit(Limit::Metric(name.into(), predicate));
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
            Limit::Metric(name, predicate) => self.until_metric(&name, predicate),
            Limit::Combined(lims) => lims
                .into_iter()
                .fold(self, |runtime, limit| runtime.limit(limit)),
        }
    }

    pub fn take(self, count: usize) -> EngineRuntime<E> {
        self.until_generation(count)
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

    #[cfg(feature = "serde")]
    pub fn checkpoint(mut self, interval: usize, folder_path: impl AsRef<Path>) -> EngineRuntime<E>
    where
        E: Engine + 'static,
        E::Epoch: Serialize,
    {
        use crate::actions::CheckpointAction;

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

    #[cfg(feature = "serde")]
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
        use crate::actions::CheckpointAction;

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

impl<E> Iterator for EngineRuntime<E>
where
    E: Engine + 'static,
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
