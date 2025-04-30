use radiate_core::{Audit, Chromosome, Ecosystem, EngineStep, MetricSet};
use std::sync::Arc;

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
            for metric in audit.audit(generation, &ecosystem) {
                metrics.upsert(metric);
            }
        }
    }
}
