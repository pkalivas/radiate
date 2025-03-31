use super::{Crossover, Mutate};
use crate::{Chromosome, Metric, Population, Rate};
use std::collections::HashSet;

/// The `AlterAction` enum is used to represent the different
/// types of alterations that can be performed on a
/// population - It can be either a mutation or a crossover operation.
pub enum Alterer<C: Chromosome> {
    Mutate(&'static str, Rate, Box<dyn Mutate<C>>),
    Crossover(&'static str, Rate, Box<dyn Crossover<C>>),
    // Adaptive(Rate, Box<Alterer<C>>),
}

impl<C: Chromosome> Alterer<C> {
    pub fn alterer(self) -> Alterer<C>
    where
        Self: Sized + 'static,
    {
        self
    }

    pub fn name(&self) -> &'static str {
        match self {
            Alterer::Mutate(name, _, _) => name,
            Alterer::Crossover(name, _, _) => name,
            // Alterer::Adaptive(_, alterer) => alterer.name(),
        }
    }

    pub fn rate(&self) -> &Rate {
        match self {
            Alterer::Mutate(_, rate, _) => rate,
            Alterer::Crossover(_, rate, _) => rate,
            // Alterer::Adaptive(rate, _) => rate,
        }
    }

    pub fn alter(&self, population: &mut Population<C>) -> AlterResult {
        match &self {
            Alterer::Mutate(name, param, m) => {
                // self.alter_with(name, population, param, |pop, rate| m.mutate(pop, rate))

                let timer = std::time::Instant::now();
                let AlterResult(count, metrics, ids) = m.mutate(population, param.get());
                let metric = Metric::new_operations(name, count, timer.elapsed());

                let result = match metrics {
                    Some(metrics) => metrics.into_iter().chain(vec![metric]).collect(),
                    None => vec![metric],
                };

                AlterResult(count, Some(result), ids)
            }
            Alterer::Crossover(name, param, c) => {
                // self.alter_with(name, population, param, |pop, rate| c.crossover(pop, rate))
                let timer = std::time::Instant::now();
                let AlterResult(count, metrics, ids) = c.crossover(population, param.get());
                let metric = Metric::new_operations(name, count, timer.elapsed());

                let result = match metrics {
                    Some(metrics) => metrics.into_iter().chain(vec![metric]).collect(),
                    None => vec![metric],
                };

                AlterResult(count, Some(result), ids)
            } // Alterer::Adaptive(rate, alterer) => {
              //     alterer.alter_with(self.name(), population, rate, |pop, r| {
              //         match alterer.as_ref() {
              //             Alterer::Mutate(_, _, m) => m.mutate(pop, r),
              //             Alterer::Crossover(_, _, c) => c.crossover(pop, r),
              //             Alterer::Adaptive(_, inner) => {
              //                 inner.alter_with(self.name(), pop, rate, |p, _| inner.alter(p))
              //             }
              //         }
              //     })
              // }
        }
    }

    fn alter_with<F: Fn(&mut Population<C>, f32) -> AlterResult>(
        &self,
        name: &'static str,
        population: &mut Population<C>,
        rate: &Rate,
        alter_fn: F,
    ) -> AlterResult {
        let timer = std::time::Instant::now();
        let AlterResult(count, metrics, ids) = alter_fn(population, rate.get());
        let metric = Metric::new_operations(name, count, timer.elapsed());

        let result = match metrics {
            Some(metrics) => metrics.into_iter().chain(vec![metric]).collect(),
            None => vec![metric],
        };

        AlterResult(count, Some(result), ids)
    }
}

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

/// The `AlterResult` struct is used to represent the result of an
/// alteration operation. It contains the number of operations
/// performed and a vector of metrics that were collected
/// during the alteration process.
#[derive(Default)]
pub struct AlterResult(pub usize, pub Option<Vec<Metric>>, pub HashSet<usize>);

impl AlterResult {
    pub fn count(&self) -> usize {
        self.0
    }

    pub fn add_count(&mut self, count: usize) {
        self.0 += count;
    }

    pub fn metrics(&self) -> Option<&Vec<Metric>> {
        self.1.as_ref()
    }

    pub fn take_metrics(&mut self) -> Option<Vec<Metric>> {
        self.1.take()
    }

    pub fn changed(&self) -> impl Iterator<Item = &usize> {
        self.2.iter()
    }

    pub fn mark_changed(&mut self, id: usize) {
        self.2.insert(id);
    }

    pub fn merge(&mut self, other: AlterResult) {
        let AlterResult(other_count, other_metrics, other_ids) = other;

        self.0 += other_count;
        if let Some(metrics) = other_metrics {
            if let Some(self_metrics) = &mut self.1 {
                self_metrics.extend(metrics);
            } else {
                self.1 = Some(metrics);
            }
        }

        self.2.extend(other_ids);
    }

    pub fn add_metric(&mut self, metric: Metric) {
        if let Some(metrics) = &mut self.1 {
            metrics.push(metric);
        } else {
            self.1 = Some(vec![metric]);
        }
    }
}

impl Into<AlterResult> for usize {
    fn into(self) -> AlterResult {
        AlterResult(self, None, HashSet::new())
    }
}

impl Into<AlterResult> for (usize, Vec<Metric>) {
    fn into(self) -> AlterResult {
        AlterResult(self.0, Some(self.1), HashSet::new())
    }
}

impl Into<AlterResult> for (usize, Metric) {
    fn into(self) -> AlterResult {
        AlterResult(self.0, Some(vec![self.1]), HashSet::new())
    }
}
