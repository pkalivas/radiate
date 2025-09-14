use crate::{Context, EngineEvent, EventBus, steps::EngineStep};
use radiate_core::Chromosome;

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
    pub fn run<T>(&mut self, context: &mut Context<C, T>, bus: &EventBus<EngineEvent<T>>)
    where
        T: Send + Sync + 'static,
    {
        context.epoch_metrics.clear();

        for step in self.steps.iter_mut() {
            bus.emit(EngineEvent::step_start(step.name()));
            let timer = std::time::Instant::now();
            step.execute(
                context.index,
                &mut context.epoch_metrics,
                &mut context.ecosystem,
            );
            bus.emit(EngineEvent::step_complete(step.name()));

            context.epoch_metrics.upsert(step.name(), timer.elapsed());
        }
    }
}

impl<C: Chromosome> Default for Pipeline<C> {
    fn default() -> Self {
        Pipeline { steps: Vec::new() }
    }
}

impl<C: Chromosome> From<Vec<Option<Box<dyn EngineStep<C>>>>> for Pipeline<C> {
    fn from(steps: Vec<Option<Box<dyn EngineStep<C>>>>) -> Self {
        let mut pipeline = Pipeline::default();
        for step in steps {
            pipeline.add_step(step);
        }
        pipeline
    }
}
