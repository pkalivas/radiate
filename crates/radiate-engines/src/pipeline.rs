use crate::{context::Context, steps::EngineStep};
use radiate_core::{Chromosome, MetricScope, Rollup, metric, metric_names};
use radiate_error::Result;

/// A [Pipeline] is a sequence of steps that are executed in order during each epoch of the engine.
/// Each step is represented by an implementation of the [EngineStep] trait.
/// The pipeline is responsible for managing the execution of these steps,
/// ensuring that they are run in the correct order and that the necessary data is passed between them.
pub(crate) struct Pipeline<C>
where
    C: Chromosome,
{
    steps: Vec<Box<dyn EngineStep<C>>>,
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

    #[inline]
    pub fn run<T>(&mut self, context: &mut Context<C, T>) -> Result<()>
    where
        C: Chromosome,
        T: Clone + Send + Sync + 'static,
    {
        context.epoch_metrics.clear();

        let timer = std::time::Instant::now();

        for step in self.steps.iter_mut() {
            let timer = std::time::Instant::now();
            step.execute(
                context.index,
                &mut context.epoch_metrics,
                &mut context.ecosystem,
            )?;
            let elapsed = timer.elapsed();

            context.epoch_metrics.add_or_update(
                metric!(MetricScope::Step, step.name(), elapsed).with_rollup(Rollup::Last),
            );
        }

        let elapsed = timer.elapsed();
        context.epoch_metrics.upsert(metric_names::TIME, elapsed);
        context.epoch_metrics.flush_all_into(&mut context.metrics);

        Ok(())
    }
}

impl<C: Chromosome> Default for Pipeline<C> {
    fn default() -> Self {
        Pipeline { steps: Vec::new() }
    }
}
