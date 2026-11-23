use std::time::Duration;

use crate::context::EvolutionContext;
use radiate_core::{Chromosome, SmallStr};
use radiate_error::Result;

type EvolutionAction<C, T> = Box<dyn FnMut(&mut EvolutionContext<C, T>) -> Result<()> + Send>;
type TimedFn<C, T> = Box<dyn FnMut(&Duration, &mut EvolutionContext<C, T>) -> Result<()> + Send>;

struct TimedFunc<C: Chromosome, T>(EvolutionAction<C, T>);

pub(crate) struct EvolutionStep<C: Chromosome, T> {
    pub(crate) name: SmallStr,
    pub(crate) action: EvolutionAction<C, T>,
}

impl<C: Chromosome, T> EvolutionStep<C, T> {
    pub fn new(name: impl Into<SmallStr>, action: EvolutionAction<C, T>) -> Self {
        Self {
            name: name.into(),
            action,
        }
    }
}

// /// A [Pipeline] is a sequence of steps that are executed in order during each epoch of the engine.
// /// Each step is represented by an implementation of the [EngineStep] trait.
// /// The pipeline is responsible for managing the execution of these steps,
// /// ensuring that they are run in the correct order and that the necessary data is passed between them.
// pub(crate) struct Pipeline<C: Chromosome, T> {
//     steps: Vec<Box<dyn EngineStep<C, T>>>,
//     names: Vec<SmallStr>,
// }

// impl<C: Chromosome, T> Pipeline<C, T> {
//     pub fn add_step(&mut self, step: Option<Box<dyn EngineStep<C, T>>>) {
//         if let Some(step) = step {
//             let first_part = step.name();
//             let name = SmallStr::from_string(format!("{}.time", first_part));
//             self.names.push(name);
//             self.steps.push(step);
//         }
//     }

//     #[inline]
//     pub fn run(&mut self, context: &mut EvolutionContext<C, T>) -> Result<()> {
//         let timer = std::time::Instant::now();

//         for (step, name) in self.steps.iter_mut().zip(self.names.iter()) {
//             let timer = std::time::Instant::now();
//             step.execute(context.index, &mut context.ecosystem, &mut context.metrics)?;
//             let elapsed = timer.elapsed();

//             context.metrics.upsert(name, elapsed);
//         }

//         let elapsed = timer.elapsed();
//         context.metrics.upsert(metric_names::TIME, elapsed);

//         Ok(())
//     }
// }

// impl<C: Chromosome, T> Default for Pipeline<C, T> {
//     fn default() -> Self {
//         Pipeline {
//             steps: Vec::new(),
//             names: Vec::new(),
//         }
//     }
// }
