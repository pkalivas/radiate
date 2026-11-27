use crate::{
    Chromosome, EngineStart, EngineStop, EpochComplete, EpochStart, GenerationSnapshot,
    Improvement, ThreadSync, events::EngineStateChange,
};
use crate::{builder::config::EngineConfig, events::EventStream};
use radiate_core::error::RadiateResult;
use radiate_core::{
    Ecosystem, EngineState, Front, Gate, MetricSet, Objective, Phenotype, Problem, Score,
    metric_names,
};
use radiate_core::{ExprSet, MetricUpdate};
use std::{
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

pub struct EvolutionContext<C: Chromosome, T> {
    pub(crate) ecosystem: Ecosystem<C>,
    pub(crate) best: T,
    pub(crate) index: usize,
    pub(crate) metrics: MetricSet,
    pub(crate) objective: Objective,
    pub(crate) sync: ThreadSync,
    pub(crate) score: Option<Score>,
    pub(crate) previous_score: Option<Score>,
    pub(crate) front: Arc<RwLock<Front<Phenotype<C>>>>,
    pub(crate) problem: Arc<dyn Problem<C, T>>,
    pub(crate) exprs: Option<Arc<Mutex<ExprSet>>>,
    pub(crate) stream: EventStream,
    pub(crate) state: EngineState,
}

impl<C: Chromosome, T> EvolutionContext<C, T> {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn metrics(&self) -> &MetricSet {
        &self.metrics
    }

    pub fn score(&self) -> Option<&Score> {
        self.score.as_ref()
    }

    pub fn ecosystem(&self) -> &Ecosystem<C> {
        &self.ecosystem
    }

    pub fn front(&self) -> Arc<RwLock<Front<Phenotype<C>>>> {
        Arc::clone(&self.front)
    }

    pub fn stream(&self) -> &EventStream {
        &self.stream
    }

    pub fn sync(&self) -> ThreadSync {
        self.sync.clone()
    }

    pub fn state(&self) -> EngineState {
        self.state
    }

    pub(crate) fn upsert<'a>(&mut self, name: impl AsRef<str>, value: impl Into<MetricUpdate<'a>>) {
        self.metrics.upsert(name, value);
    }

    pub(crate) fn set_running(&mut self) {
        if self.index == 0 && self.state == EngineState::PreStart {
            self.metrics.upsert(metric_names::INDEX, self.index);
        }

        self.change_state(EngineState::Running);
    }

    pub(crate) fn set_paused(&mut self) {
        self.change_state(EngineState::Paused);
    }

    pub(crate) fn set_stopped(&mut self) {
        self.change_state(EngineState::Stopped);
    }

    pub(crate) fn start(&mut self) -> RadiateResult<()>
    where
        C: Chromosome + Clone + 'static,
        T: Clone + Send + Sync + 'static,
    {
        if self.state == EngineState::PreStart {
            self.stream.open()?;
            self.set_running();
            self.stream.lazy_publish(|| EngineStart::from(&*self))?;
        }

        Ok(())
    }

    pub(crate) fn stop(&mut self) -> RadiateResult<()>
    where
        C: Chromosome + Clone + 'static,
        T: Clone + Send + Sync + 'static,
    {
        self.set_stopped();
        self.stream.lazy_publish(|| EngineStop::from(&*self))?;
        self.stream.close()
    }

    pub(crate) fn process(
        &mut self,
        mut scope: impl FnMut(&mut Self) -> RadiateResult<Duration>,
    ) -> RadiateResult<()>
    where
        C: Chromosome + Clone + 'static,
        T: Clone + Send + Sync + 'static,
    {
        if !self.can_continue() {
            return self.stop();
        }

        self.stream.lazy_publish(|| EpochStart::from(&*self))?;

        let duration = scope(self)?;

        let best = self.ecosystem.get_phenotype(0);
        let best_improved = self
            .metrics
            .improvements()
            .map(|m| m.last_value() > 0.0)
            .unwrap_or(false);

        if best_improved && let Some(best) = best {
            self.previous_score = self.score.clone();
            self.score = best.score().cloned();
            self.best = self.problem.decode(best.genotype());

            self.stream.lazy_publish(|| Improvement::from(&*self))?;
        }

        self.index += 1;
        self.metrics.replace(metric_names::INDEX, self.index);
        self.metrics.upsert(metric_names::TIME, duration);
        self.metrics.bump(self.index);

        self.stream.lazy_publish(|| EpochComplete::from(&*self))?;
        self.stream
            .lazy_publish(|| GenerationSnapshot::from(&*self))?;
        self.stream.roll()
    }

    fn can_continue(&mut self) -> bool {
        match self.sync.gate() {
            Gate::Open => true,
            Gate::Stopped => false,
            Gate::Closed => {
                // If sync.gate() returned Closed, the sync was paused from an external
                // request, so we change the state to paused then park.
                self.set_paused();
                if self.sync.wait().is_continue() {
                    self.set_running();
                    return true;
                }

                false
            }
        }
    }

    fn change_state(&mut self, state: EngineState) {
        if self.state == state {
            return;
        }

        self.stream
            .lazy_publish(|| EngineStateChange {
                from: self.state,
                to: state,
                index: self.index,
            })
            .unwrap();

        self.state = state;
    }
}

impl<C, T> From<EngineConfig<C, T>> for EvolutionContext<C, T>
where
    C: Chromosome + Clone,
    T: Clone,
{
    fn from(config: EngineConfig<C, T>) -> Self {
        if let Some(generation) = config.generation() {
            return EvolutionContext {
                ecosystem: generation.ecosystem().clone(),
                best: generation.value().clone(),
                index: generation.index(),
                metrics: generation.metrics().clone(),
                score: Some(generation.score().clone()),
                previous_score: None,
                front: config.front(),
                objective: config.objective().clone(),
                problem: config.problem().clone(),
                sync: config.sync(),
                exprs: generation.exprs(),
                stream: config.event_stream(),
                state: EngineState::PreStart,
            };
        }

        let initial_genotype = config
            .ecosystem()
            .get_genotype(0)
            .map(|geno| config.problem().decode(geno));

        EvolutionContext {
            ecosystem: config.ecosystem().clone(),
            best: initial_genotype.unwrap(),
            index: 0,
            metrics: MetricSet::default(),
            score: None,
            previous_score: None,
            front: config.front(),
            objective: config.objective().clone(),
            problem: config.problem().clone(),
            sync: config.sync(),
            exprs: config.exprs(),
            stream: config.event_stream(),
            state: EngineState::PreStart,
        }
    }
}
