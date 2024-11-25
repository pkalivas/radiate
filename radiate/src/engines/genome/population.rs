use super::phenotype::Phenotype;
use crate::Chromosome;
use std::fmt::Debug;

/// A `Population` is a collection of `Phenotype` instances. This struct is the core collection of individuals
/// being evolved by the `GeneticEngine`. It can be thought of as a Vec of `Phenotype`s and
/// is essentially a light wrapper around such a Vec. The `Population` struct, however, has some
/// additional functionality that allows for sorting and iteration over the individuals in the population.
///
/// Note: Although the `Population` offers mut methods to mut the individuals in the population, the `Population`
/// itself offers no way to increase or decrease the number of individuals in the population. As such, the `Population`
/// should be thought of as an 'immutable' data structure. If you need to add or remove individuals from the population,
/// you should create a new `Population` instance with the new individuals. To further facilitate this way of
/// thinking, the `Population` struct and everyhing it contains implements the `Clone` trait.
///
/// # Type Parameters
/// - `C`: The type of chromosome used in the genotype, which must implement the `Chromosome` trait.
///
#[derive(Clone)]
pub struct Population<C: Chromosome> {
    pub individuals: Vec<Phenotype<C>>,
    pub is_sorted: bool,
}

impl<C: Chromosome> Population<C> {
    /// Create a new instance of the Population. This will create a new instance with an
    /// empty list of individuals and the is_sorted flag set to false.
    pub fn new() -> Self {
        Population {
            individuals: Vec::new(),
            is_sorted: false,
        }
    }

    pub fn get(&self, index: usize) -> &Phenotype<C> {
        self.individuals.get(index).expect("Index out of bounds")
    }

    /// Get a mutable reference to the individual at the given index. This will set the is_sorted flag to false
    /// because we cannot guarantee that the individual's `Score` (fitness) has not changed.
    pub fn get_mut(&mut self, index: usize) -> &mut Phenotype<C> {
        self.is_sorted = false;
        self.individuals
            .get_mut(index)
            .expect("Index out of bounds")
    }

    /// Set the individual at the given index. This will set the is_sorted flag to false
    /// because we cannot guarantee that the individual is in the correct order.
    pub fn set(&mut self, index: usize, individual: Phenotype<C>) {
        self.individuals[index] = individual;
        self.is_sorted = false;
    }

    pub fn iter(&self) -> std::slice::Iter<Phenotype<C>> {
        self.individuals.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<Phenotype<C>> {
        self.is_sorted = false;
        self.individuals.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.individuals.len()
    }

    /// Sort the individuals in the population using the given closure. This will set the is_sorted flag to true.
    pub fn sort_by<F>(&mut self, f: F)
    where
        F: FnMut(&Phenotype<C>, &Phenotype<C>) -> std::cmp::Ordering,
    {
        if self.is_sorted {
            return;
        }

        self.individuals.sort_by(f);
        self.is_sorted = true;
    }

    pub fn from_vec(individuals: Vec<Phenotype<C>>) -> Self {
        Population {
            individuals,
            is_sorted: false,
        }
    }

    pub fn from_fn<F>(size: usize, f: F) -> Self
    where
        F: Fn() -> Phenotype<C>,
    {
        let mut individuals = Vec::with_capacity(size);
        for _ in 0..size {
            individuals.push(f());
        }

        Population {
            individuals,
            is_sorted: false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.individuals.is_empty()
    }
}

impl<C: Chromosome> Default for Population<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Chromosome> IntoIterator for Population<C> {
    type Item = Phenotype<C>;
    type IntoIter = std::vec::IntoIter<Phenotype<C>>;

    fn into_iter(self) -> Self::IntoIter {
        self.individuals.into_iter()
    }
}

impl<C: Chromosome> FromIterator<Phenotype<C>> for Population<C> {
    fn from_iter<I: IntoIterator<Item = Phenotype<C>>>(iter: I) -> Self {
        let individuals = iter.into_iter().collect();
        Population {
            individuals,
            is_sorted: false,
        }
    }
}

impl<C: Chromosome + Debug> Debug for Population<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for individual in &self.individuals {
            write!(f, "{:?},\n ", individual)?;
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::CharChromosome;

    #[test]
    fn test_new() {
        let population = Population::<CharChromosome>::new();
        assert_eq!(population.len(), 0);
    }
}
