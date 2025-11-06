use crate::steps::EngineStep;
use radiate_core::{Chromosome, Ecosystem, Evaluator, MetricSet, Objective, Problem, metric_names};
use radiate_error::Result;
use std::sync::Arc;

pub struct EvaluateStep<C: Chromosome, T> {
    pub(crate) objective: Objective,
    pub(crate) evaluator: Arc<dyn Evaluator<C, T>>,
    pub(crate) problem: Arc<dyn Problem<C, T>>,
}

impl<C: Chromosome, T> EvaluateStep<C, T> {
    pub fn new(
        objective: Objective,
        problem: Arc<dyn Problem<C, T>>,
        evaluator: Arc<dyn Evaluator<C, T>>,
    ) -> Self {
        Self {
            objective,
            evaluator,
            problem,
        }
    }
}

impl<C, T> EngineStep<C> for EvaluateStep<C, T>
where
    C: Chromosome + PartialEq,
{
    #[inline]
    fn execute(
        &mut self,
        _: usize,
        metrics: &mut MetricSet,
        ecosystem: &mut Ecosystem<C>,
    ) -> Result<()> {
        let count = self.evaluator.eval(ecosystem, Arc::clone(&self.problem))?;

        self.objective.sort(&mut ecosystem.population);

        if count > 0 {
            metrics.upsert(metric_names::EVALUATION_COUNT, count);
        }

        Ok(())
    }
}
