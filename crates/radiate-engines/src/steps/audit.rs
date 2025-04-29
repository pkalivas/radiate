use std::sync::Arc;

use radiate_core::{Audit, Chromosome, Ecosystem, EngineStep, MetricSet};

pub struct AuditStep<C>
where
    C: Chromosome,
{
    pub(crate) audits: Vec<Arc<dyn Audit<C>>>,
}

impl<C> EngineStep<C> for AuditStep<C>
where
    C: Chromosome,
{
    fn execute(
        &mut self,
        generation: usize,
        metrics: &mut MetricSet,
        ecosystem: &mut Ecosystem<C>,
    ) {
        for audit in &self.audits {
            for metric in audit.audit(generation, &ecosystem.population) {
                metrics.upsert(metric);
            }
        }
    }
}
