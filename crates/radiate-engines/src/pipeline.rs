use crate::{context::Context, steps::EngineStep};
use radiate_core::{Chromosome, MetricSet, metric, metric_names};
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
    metrics: MetricSet,
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
    {
        let timer = std::time::Instant::now();

        for step in self.steps.iter_mut() {
            let timer = std::time::Instant::now();
            step.execute(context.index, &mut context.ecosystem, &mut self.metrics)?;
            let elapsed = timer.elapsed();

            self.metrics.upsert(metric!(step.name(), elapsed));
        }

        let elapsed = timer.elapsed();

        self.metrics.flush_all_into(&mut context.metrics);
        context.metrics.upsert((metric_names::TIME, elapsed));

        Ok(())
    }
}

impl<C: Chromosome> Default for Pipeline<C> {
    fn default() -> Self {
        Pipeline {
            steps: Vec::new(),
            metrics: MetricSet::new(),
        }
    }
}
