use std::sync::Arc;

use radiate_core::{Audit, Chromosome, EngineStep};

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
        &self,
        generation: usize,
        metrics: &mut radiate_core::MetricSet,
        ecosystem: &mut radiate_core::Ecosystem<C>,
    ) {
        for audit in &self.audits {
            for metric in audit.audit(generation, &ecosystem.population) {
                metrics.upsert(metric);
            }
        }
    }
}
