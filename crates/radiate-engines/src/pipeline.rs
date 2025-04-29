use radiate_core::{Chromosome, Ecosystem, EngineStep, MetricSet, timer::Timer};

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

    pub fn run(&self, generation: usize, metrics: &mut MetricSet, ecosystem: &mut Ecosystem<C>) {
        for step in &self.steps {
            let timer = Timer::new();
            step.execute(generation, metrics, ecosystem);
            let duration = timer.duration();
            metrics.upsert_time(step.name(), duration);
        }
    }
}

impl<C: Chromosome> Default for Pipeline<C> {
    fn default() -> Self {
        Pipeline { steps: Vec::new() }
    }
}
