use std::sync::Arc;

use crate::{Audit, Chromosome, EngineContext, GeneticEngineParams, Genotype, Metric, Objective};

use super::EngineStep;

pub struct AuditStep<C: Chromosome, T> {
    audits: Vec<Arc<dyn Audit<C>>>,
    decoder: Arc<dyn Fn(&Genotype<C>) -> T>,
    objective: Objective,
}

/// Audits the current state of the genetic algorithm, updating the best individual found so far
/// and calculating various metrics such as the age of individuals, the score of individuals, and the
/// number of unique scores in the population. This method is called at the end of each generation.
impl<C, T> EngineStep<C, T> for AuditStep<C, T>
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
        let problem = params.problem();
        let decoder = Arc::new(move |genotype: &Genotype<C>| problem.decode(genotype));

        Some(Box::new(AuditStep {
            audits,
            decoder,
            objective: params.objective().clone(),
        }))
    }

    fn execute(&self, ctx: &mut EngineContext<C, T>) {
        let audit_metrics = self
            .audits
            .iter()
            .map(|audit| audit.audit(ctx.index(), &ctx.population))
            .flatten()
            .collect::<Vec<Metric>>();

        for metric in audit_metrics {
            ctx.record_metric(metric);
        }

        if !ctx.population.is_sorted {
            self.objective.sort(&mut ctx.population);
        }

        let current_best = ctx.population.get(0);

        if let (Some(best), Some(current)) = (current_best.score(), &ctx.score) {
            if self.objective.is_better(&best, &current) {
                ctx.score = Some(best.clone());
                ctx.best = (self.decoder)(&current_best.genotype());
            }
        } else {
            ctx.score = Some(current_best.score().unwrap().clone());
            ctx.best = (self.decoder)(&current_best.genotype());
        }

        ctx.index += 1;
    }
}
