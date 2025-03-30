use super::EngineStep;
use crate::{Audit, Chromosome, GeneticEngineParams, Metric, Objective, Population, Species};
use std::sync::Arc;

pub struct AuditStep<C: Chromosome> {
    audits: Vec<Arc<dyn Audit<C>>>,
    objective: Objective,
}

/// Audits the current state of the genetic algorithm, updating the best individual found so far
/// and calculating various metrics such as the age of individuals, the score of individuals, and the
/// number of unique scores in the population. This method is called at the end of each generation.
impl<C, T> EngineStep<C, T> for AuditStep<C>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    fn register(params: &GeneticEngineParams<C, T>) -> Option<Box<Self>>
    where
        Self: Sized,
    {
        let audits = params
            .audits()
            .iter()
            .map(|audit| Arc::clone(audit))
            .collect::<Vec<Arc<dyn Audit<C>>>>();

        Some(Box::new(AuditStep {
            audits,
            objective: params.objective().clone(),
        }))
    }

    fn execute(
        &self,
        generation: usize,
        population: &mut Population<C>,
        _: &mut Vec<Species<C>>,
    ) -> Vec<Metric> {
        let audit_metrics = self
            .audits
            .iter()
            .map(|audit| audit.audit(generation, &population))
            .flatten()
            .collect::<Vec<Metric>>();

        if !population.is_sorted {
            self.objective.sort(population);
        }

        return audit_metrics;
    }
}
