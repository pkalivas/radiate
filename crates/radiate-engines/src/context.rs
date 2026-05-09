use crate::builder::config::EngineConfig;
use crate::{Chromosome, EngineControl};
use radiate_core::error::RadiateResult;
use radiate_core::stats::TagType;
use radiate_core::{
    Ecosystem, Front, Lineage, MetricSet, MetricUpdate, Objective, Phenotype, Problem,
    RadiateError, Score, metric, metric_names,
};
use radiate_expr::{Evaluate, NamedExpr};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, RwLock};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub enum ContextAudit {
    NewBest,
    LimitReached(String),
}

pub struct Context<C: Chromosome, T> {
    pub(crate) ecosystem: Ecosystem<C>,
    pub(crate) best: T,
    pub(crate) index: usize,
    pub(crate) metrics: MetricSet,
    pub(crate) score: Option<Score>,
    pub(crate) front: Arc<RwLock<Front<Phenotype<C>>>>,
    pub(crate) lineage: Arc<RwLock<Lineage>>,
    pub(crate) objective: Objective,
    pub(crate) problem: Arc<dyn Problem<C, T>>,
    pub(crate) control: Option<EngineControl>,
    pub(crate) exprs: Option<Arc<Mutex<Vec<NamedExpr>>>>,
    pub(crate) audits: Vec<ContextAudit>,
}

impl<C: Chromosome, T> Context<C, T> {
    pub fn try_advance_one(&mut self) -> RadiateResult<bool> {
        self.index += 1;
        self.lineage.write().unwrap().rollover();
        self.audits.clear();

        self.metrics
            .replace(metric!(metric_names::INDEX, self.index));

        let mut best_improved = false;

        let best = self.ecosystem.get_phenotype(0);
        if let Some(best) = best {
            if let (Some(score), Some(current)) = (best.score(), &self.score) {
                if !self.objective.validate(score) {
                    return Err(RadiateError::Fitness(format!(
                        "Score {:?} has invalid dimensions for the objective {:?}.",
                        score, self.objective
                    )));
                }

                if self.objective.is_better(score, current) {
                    self.score = Some(score.clone());
                    self.best = self.problem.decode(best.genotype());

                    best_improved = true;
                }
            } else {
                self.score = best.score().cloned();
                self.best = self.problem.decode(best.genotype());

                best_improved = true;
            }
        }

        if best_improved {
            self.metrics
                .upsert((metric_names::BEST_SCORE_IMPROVEMENT, 1));
            self.audits.push(ContextAudit::NewBest);
        }

        if let Some(score) = &self.score {
            if score.len() == 1 {
                self.metrics.upsert((metric_names::BEST_SCORES, score[0]));
            } else {
                for (i, score) in score.as_slice().iter().enumerate() {
                    self.metrics.upsert((metric_names::BEST_SCORES, *score, i));
                }
            }
        }

        if let Some(exprs) = &self.exprs {
            let mut exprs = exprs.lock().unwrap();
            for expr in exprs.iter_mut() {
                let (name, exp) = expr.pair();

                let update = MetricUpdate::try_from(exp.eval(&self.metrics)?)?;
                let name = radiate_utils::intern!(name);

                self.metrics.upsert((TagType::Expr, name, update));
            }
        }

        self.metrics.next_version();

        Ok(best_improved)
    }

    pub fn get_or_create_control(&mut self) -> EngineControl {
        if self.control.is_none() {
            let (one, two) = EngineControl::pair();
            self.control = Some(one);
            return two;
        }

        self.control.as_ref().unwrap().clone()
    }
}

impl<C, T> From<EngineConfig<C, T>> for Context<C, T>
where
    C: Chromosome + Clone,
    T: Clone,
{
    fn from(config: EngineConfig<C, T>) -> Self {
        if let Some(generation) = config.generation() {
            return Context {
                ecosystem: generation.ecosystem().clone(),
                best: generation.value().clone(),
                index: generation.index(),
                metrics: generation.metrics().clone(),
                score: Some(generation.score().clone()),
                front: config.front(),
                lineage: config.lineage(),
                objective: config.objective().clone(),
                problem: config.problem().clone(),
                control: None,
                exprs: generation.exprs(),
                audits: vec![],
            };
        }

        let initial_genotype = config
            .ecosystem()
            .get_genotype(0)
            .map(|geno| config.problem().decode(geno));

        Context {
            ecosystem: config.ecosystem().clone(),
            best: initial_genotype.unwrap(),
            index: 0,
            metrics: MetricSet::default(),
            score: None,
            front: config.front(),
            lineage: config.lineage(),
            objective: config.objective().clone(),
            problem: config.problem().clone(),
            control: None,
            exprs: config.exprs(),
            audits: vec![],
        }
    }
}
