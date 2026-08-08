use crate::{Chromosome, ThreadSync};
use crate::{builder::config::EngineConfig, message::EventStream};
use radiate_core::error::RadiateResult;
use radiate_core::rate::ExprSet;
use radiate_core::{
    Ecosystem, Front, MetricSet, Objective, Phenotype, Problem, Score, metric, metric_names,
};
use std::sync::{Arc, Mutex, RwLock};

pub struct EvolutionContext<C: Chromosome, T> {
    pub(crate) ecosystem: Ecosystem<C>,
    pub(crate) best: T,
    pub(crate) index: usize,
    pub(crate) metrics: MetricSet,
    pub(crate) objective: Objective,
    pub(crate) sync: ThreadSync,
    pub(crate) score: Option<Score>,
    pub(crate) front: Arc<RwLock<Front<Phenotype<C>>>>,
    pub(crate) problem: Arc<dyn Problem<C, T>>,
    pub(crate) exprs: Option<Arc<Mutex<ExprSet>>>,
    pub(crate) events: EventStream,
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
        self.front.clone()
    }

    pub fn events(&self) -> &EventStream {
        &self.events
    }

    pub fn is_stopped(&self) -> bool {
        self.sync.is_stopped()
    }

    pub fn stop(&mut self) {
        self.sync.stop();
    }

    pub fn is_paused(&self) -> bool {
        self.sync.is_paused()
    }

    pub fn wait(&self) {
        self.sync.wait()
    }

    pub fn get_or_create_control(&mut self) -> ThreadSync {
        self.sync.clone()
    }

    pub(crate) fn try_advance_one(&mut self) -> RadiateResult<bool> {
        self.index += 1;

        let best = self.ecosystem.get_phenotype(0);
        let best_improved = self
            .metrics
            .improvements()
            .map(|m| m.last_value() > 0.0)
            .unwrap_or(false);

        if best_improved && let Some(best) = best {
            self.score = best.score().cloned();
            self.best = self.problem.decode(best.genotype());
        }

        self.metrics
            .replace(metric!(metric_names::INDEX, self.index));
        self.metrics.bump(self.index as u64);

        Ok(best_improved)
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
                front: config.front(),
                objective: config.objective().clone(),
                problem: config.problem().clone(),
                sync: config.sync(),
                exprs: generation.exprs(),
                events: config.event_stream(),
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
            front: config.front(),
            objective: config.objective().clone(),
            problem: config.problem().clone(),
            sync: config.sync(),
            exprs: config.exprs(),
            events: config.event_stream(),
        }
    }
}
