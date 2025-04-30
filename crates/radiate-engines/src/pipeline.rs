use radiate_core::{Chromosome, Ecosystem, EngineStep, MetricSet};

pub struct Pipeline<C>
where
    C: Chromosome,
{
    pub steps: Vec<Box<dyn EngineStep<C>>>,
}

impl<C> Pipeline<C>
where
    C: Chromosome,
{
    pub fn new(steps: Vec<Box<dyn EngineStep<C>>>) -> Self {
        Pipeline { steps }
    }

    pub fn add_step(&mut self, step: Option<Box<dyn EngineStep<C>>>) {
        if let Some(step) = step {
            self.steps.push(step);
        }
    }

    pub fn run(
        &mut self,
        generation: usize,
        metrics: &mut MetricSet,
        ecosystem: &mut Ecosystem<C>,
    ) {
        for step in self.steps.iter_mut() {
            let timer = std::time::Instant::now();
            step.execute(generation, metrics, ecosystem);

            metrics.upsert_time(step.name(), timer.elapsed());
        }
    }
}

impl<C: Chromosome> Default for Pipeline<C> {
    fn default() -> Self {
        Pipeline { steps: Vec::new() }
    }
}
