use crate::{context::Context, steps::EngineStep};
use radiate_core::{Chromosome, metric_names};
use radiate_error::Result;

/// A [Pipeline] is a sequence of steps that are executed in order during each epoch of the engine.
/// Each step is represented by an implementation of the [EngineStep] trait.
/// The pipeline is responsible for managing the execution of these steps,
/// ensuring that they are run in the correct order and that the necessary data is passed between them.
pub(crate) struct Pipeline<C: Chromosome> {
    steps: Vec<Box<dyn EngineStep<C>>>,
    names: Vec<&'static str>,
}

impl<C: Chromosome> Pipeline<C> {
    pub fn add_step(&mut self, step: Option<Box<dyn EngineStep<C>>>) {
        if let Some(step) = step {
            let first_part = step.name();
            let name = radiate_utils::intern!(format!("{}.time", first_part));
            self.names.push(name);
            self.steps.push(step);
        }
    }

    #[inline]
    pub fn run<T>(&mut self, context: &mut Context<C, T>) -> Result<()> {
        let timer = std::time::Instant::now();

        for (step, name) in self.steps.iter_mut().zip(self.names.iter()) {
            let timer = std::time::Instant::now();
            step.execute(context.index, &mut context.ecosystem, &mut context.metrics)?;
            let elapsed = timer.elapsed();

            context.metrics.upsert((*name, elapsed));
        }

        let elapsed = timer.elapsed();
        context.metrics.upsert((metric_names::TIME, elapsed));

        Ok(())
    }
}

impl<C: Chromosome> Default for Pipeline<C> {
    fn default() -> Self {
        Pipeline {
            steps: Vec::new(),
            names: Vec::new(),
        }
    }
}
