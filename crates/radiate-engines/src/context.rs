use crate::Chromosome;
use crate::builder::EngineConfig;
use radiate_core::{Ecosystem, Front, MetricSet, Objective, Phenotype, Problem, Score};
use std::sync::{Arc, RwLock};

pub struct Context<C: Chromosome, T> {
    pub(crate) ecosystem: Ecosystem<C>,
    pub(crate) best: T,
    pub(crate) index: usize,
    pub(crate) metrics: MetricSet,
    pub(crate) epoch_metrics: MetricSet,
    pub(crate) score: Option<Score>,
    pub(crate) front: Arc<RwLock<Front<Phenotype<C>>>>,
    pub(crate) objective: Objective,
    pub(crate) problem: Arc<dyn Problem<C, T>>,
}

impl<C: Chromosome, T> Context<C, T> {
    pub fn try_advance_one(&mut self) -> bool {
        self.index += 1;

        let best = self.ecosystem.population().get(0);
        if let Some(best) = best {
            if let (Some(score), Some(current)) = (best.score(), &self.score) {
                if self.objective.is_better(score, current) {
                    self.score = Some(score.clone());
                    self.best = self.problem.decode(best.genotype());
                    return true;
                }
            } else {
                self.score = Some(best.score().unwrap().clone());
                self.best = self.problem.decode(best.genotype());
                return true;
            }
        }

        false
    }
}

impl<C, T> From<EngineConfig<C, T>> for Context<C, T>
where
    C: Chromosome + Clone,
    T: Clone,
{
    fn from(config: EngineConfig<C, T>) -> Self {
        Context {
            ecosystem: Ecosystem::new(config.population().clone()),
            best: config.problem().decode(config.population()[0].genotype()),
            index: 0,
            metrics: MetricSet::default(),
            epoch_metrics: MetricSet::default(),
            score: None,
            front: config.front(),
            objective: config.objective().clone(),
            problem: config.problem().clone(),
        }
    }
}
