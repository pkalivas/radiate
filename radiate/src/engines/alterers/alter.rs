use super::{Crossover, Mutate};
use crate::{Chromosome, EngineCompoment, Metric, Population, timer::Timer};

pub enum AlterAction<C: Chromosome> {
    Mutate(Box<dyn Mutate<C>>),
    Crossover(Box<dyn Crossover<C>>),
}

pub trait Alter<C: Chromosome>: EngineCompoment {
    fn rate(&self) -> f32;
    fn to_alter(self) -> AlterAction<C>;
}

pub trait IntoAlter<C: Chromosome> {
    fn into_alter(self) -> Alterer<C>;
}

pub trait AlterFn<C: Chromosome> {
    fn alter(&self, population: &mut Population<C>, generation: i32) -> Vec<Metric>;
}

pub struct Alterer<C: Chromosome> {
    name: &'static str,
    rate: f32,
    alter: AlterAction<C>,
}

impl<C: Chromosome> Alterer<C> {
    pub fn new(name: &'static str, rate: f32, alter: AlterAction<C>) -> Self {
        Self { name, rate, alter }
    }

    pub fn name(&self) -> &'static str {
        &self.name
    }

    pub fn rate(&self) -> f32 {
        self.rate
    }

    pub fn alter(&self) -> &AlterAction<C> {
        &self.alter
    }
}

impl<C: Chromosome> AlterFn<C> for Alterer<C> {
    fn alter(&self, population: &mut Population<C>, generation: i32) -> Vec<Metric> {
        match &self.alter {
            AlterAction::Mutate(m) => {
                let timer = Timer::new();
                let result = m.mutate(population, generation, self.rate);

                let duration = timer.duration();
                let count = result.count as f32;
                let metric = Metric::new_operations(self.name(), count, duration);
                return vec![metric].into_iter().chain(result.metrics).collect();
            }
            AlterAction::Crossover(c) => {
                let timer = Timer::new();
                let crossover_result = c.crossover(population, generation, self.rate);
                let duration = timer.duration();
                let count = crossover_result.count as f32;
                let metric = Metric::new_operations(self.name(), count, duration);

                return vec![metric]
                    .into_iter()
                    .chain(crossover_result.metrics)
                    .collect();
            }
        }
    }
}
