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
    fn alter(&self, population: &mut Population<C>, generation: i32) -> Vec<Metric>;
}

/// The `IntoAlter` trait is used to convert a struct into an `Alterer` struct. Because an `Alterer`
/// can be a `Mutate` or a `Crossover`, this trait is used to combine the two into one joined trait.
/// This is mostly a convience and quality of life trait so that the user does not have to
/// manually create an `Alterer` struct from a `Mutate` or `Crossover` struct.
pub trait IntoAlter<C: Chromosome> {
    fn into_alter(self) -> Alterer<C>;
}

/// The `AlterResult` struct is used to represent the result of an
/// alteration operation. It contains the number of operations
/// performed and a vector of metrics that were collected
/// during the alteration process.
pub struct AlterResult(pub i32, pub Vec<Metric>);

/// The `AlterAction` enum is used to represent the different
/// types of alterations that can be performed on a
/// population - It can be either a mutation or a crossover operation.
pub enum AlterAction<C: Chromosome> {
    Mutate(Box<dyn Mutate<C>>),
    Crossover(Box<dyn Crossover<C>>),
}

/// The `Alterer` struct is used to represent an alterer that can
/// perform an alteration on a population. It contains a name,
/// a rate, and an `AlterAction` that represents the type of
/// alteration that will be performed. The `Alterer` struct
/// implements the `Alter` trait, which allows it to perform
/// the alteration on a population. The `Alterer` struct is a way to join together
/// the `Mutate` and `Crossover` traits into a single struct.
pub struct Alterer<C: Chromosome> {
    name: &'static str,
    rate: f32,
    alter: AlterAction<C>,
}

impl<C: Chromosome> Alterer<C> {
    /// Creates a new `Alterer` struct with the given name, rate, and `AlterAction`.
    ///
    ///  The `name` is a static string that represents the name of the alterer.
    /// The `rate` is a float that represents the rate at which the alteration will be performed.
    /// The `alter` is an `AlterAction` that represents the type of alteration that will be performed,
    /// which can be either a mutation or a crossover operation.
    pub fn new(name: &'static str, rate: f32, alter: AlterAction<C>) -> Self {
        Self { name, rate, alter }
    }
}

impl<C: Chromosome> Alter<C> for Alterer<C> {
    fn alter(&self, population: &mut Population<C>, generation: i32) -> Vec<Metric> {
        match &self.alter {
            AlterAction::Mutate(m) => {
                let timer = Timer::new();
                let AlterResult(count, metrics) = m.mutate(population, generation, self.rate);
                let metric = Metric::new_operations(self.name, count, timer);

                return vec![metric].into_iter().chain(metrics).collect();
            }
            AlterAction::Crossover(c) => {
                let timer = Timer::new();
                let AlterResult(count, metrics) = c.crossover(population, generation, self.rate);
                let metric = Metric::new_operations(self.name, count, timer);

                return vec![metric].into_iter().chain(metrics).collect();
            }
        }
    }
}
