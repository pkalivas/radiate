use super::{Crossover, Mutate};
use crate::{Chromosome, Metric, Population, timer::Timer};

/// This is the main trait that is used to define the different types of alterations that can be
/// performed on a population. The `Alter` trait is used to define the `alter` method that is used
/// to perform the alteration on the population. The `alter` method takes a mutable reference to
/// the population and a generation number as parameters. The `alter` method returns a vector of
/// `Metric` objects that represent the metrics that were collected during the alteration process.
///
/// An 'Alter' in a traditional genetic algorithm is a process that modifies the population of
/// individuals in some way. This can include operations such as mutation or crossover. The goal of
/// an alter is to introduce new genetic material into the population, which can help to improve
/// the overall fitness of the population. In a genetic algorithm, the alter is typically
/// performed on a subset of the population, rather than the entire population. This allows for
/// more targeted modifications to be made, which can help to improve the overall performance of
/// the algorithm. The alter is an important part of the genetic algorithm process, as it helps
/// to ensure that the population remains diverse and that new genetic material is introduced
/// into the population. This can help to improve the overall performance of the algorithm and
/// ensure that the population remains healthy and diverse.
///
/// In `radiate` the `alter` trait performs similar operations to a traditional genetic algorithm,
/// but it is designed to be more flexible and extensible. Because an `Alter` can be of type `Mutate`
/// or `Crossover`, it is abstracted out of those core traits into this trait.
pub trait Alter<C: Chromosome> {
    fn alter(&self, population: &mut Population<C>, generation: usize) -> Vec<Metric>;
}

/// The `AlterResult` struct is used to represent the result of an
/// alteration operation. It contains the number of operations
/// performed and a vector of metrics that were collected
/// during the alteration process.
#[derive(Default)]
pub struct AlterResult(pub usize, pub Option<Vec<Metric>>);

impl AlterResult {
    pub fn count(&self) -> usize {
        self.0
    }

    pub fn metrics(&self) -> Option<&Vec<Metric>> {
        self.1.as_ref()
    }

    pub fn merge(&mut self, other: AlterResult) {
        let AlterResult(other_count, other_metrics) = other;

        self.0 += other_count;
        if let Some(metrics) = other_metrics {
            if let Some(self_metrics) = &mut self.1 {
                self_metrics.extend(metrics);
            } else {
                self.1 = Some(metrics);
            }
        }
    }
}

impl Into<AlterResult> for usize {
    fn into(self) -> AlterResult {
        AlterResult(self, None)
    }
}

impl Into<AlterResult> for (usize, Vec<Metric>) {
    fn into(self) -> AlterResult {
        AlterResult(self.0, Some(self.1))
    }
}

impl Into<AlterResult> for (usize, Metric) {
    fn into(self) -> AlterResult {
        AlterResult(self.0, Some(vec![self.1]))
    }
}

/// The `AlterAction` enum is used to represent the different
/// types of alterations that can be performed on a
/// population - It can be either a mutation or a crossover operation.
pub enum AlterAction<C: Chromosome> {
    Mutate(&'static str, f32, Box<dyn Mutate<C>>),
    Crossover(&'static str, f32, Box<dyn Crossover<C>>),
}

impl<C: Chromosome> Alter<C> for AlterAction<C> {
    fn alter(&self, population: &mut Population<C>, generation: usize) -> Vec<Metric> {
        match &self {
            AlterAction::Mutate(name, rate, m) => {
                let timer = Timer::new();
                let AlterResult(count, metrics) = m.mutate(population, generation, *rate);
                let metric = Metric::new_operations(name, count, timer);

                match metrics {
                    Some(metrics) => metrics.into_iter().chain(std::iter::once(metric)).collect(),
                    None => vec![metric],
                }
            }
            AlterAction::Crossover(name, rate, c) => {
                let timer = Timer::new();
                let AlterResult(count, metrics) = c.crossover(population, generation, *rate);
                let metric = Metric::new_operations(name, count, timer);

                match metrics {
                    Some(metrics) => metrics.into_iter().chain(std::iter::once(metric)).collect(),
                    None => vec![metric],
                }
            }
        }
    }
}
