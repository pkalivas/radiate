use radiate_core::{Chromosome, Ecosystem, EngineStep, MetricSet};

use crate::{EngineEvent, EventBus};

pub(crate) struct Pipeline<C>
where
    C: Chromosome,
{
    pub steps: Vec<Box<dyn EngineStep<C>>>,
}

impl<C> Pipeline<C>
where
    C: Chromosome,
{
    pub fn add_step(&mut self, step: Option<Box<dyn EngineStep<C>>>) {
        if let Some(step) = step {
            self.steps.push(step);
        }
    }

    pub fn run<T>(
        &mut self,
        generation: usize,
        bus: &EventBus<EngineEvent<T>>,
        metrics: &mut MetricSet,
        ecosystem: &mut Ecosystem<C>,
    ) where
        T: Send + Sync + 'static,
    {
        for step in self.steps.iter_mut() {
            bus.emit(EngineEvent::step_start(step.name()));
            let timer = std::time::Instant::now();
            step.execute(generation, metrics, ecosystem);
            bus.emit(EngineEvent::step_complete(step.name()));

            metrics.upsert_time(step.name(), timer.elapsed());
        }
    }
}

impl<C: Chromosome> Default for Pipeline<C> {
    fn default() -> Self {
        Pipeline { steps: Vec::new() }
    }
}
