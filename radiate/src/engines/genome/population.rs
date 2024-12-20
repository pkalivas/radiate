use super::phenotype::Phenotype;
use crate::objectives::score::Score;
use crate::Chromosome;
use std::fmt::Debug;
use std::ops::{Index, IndexMut};

/// A `Population` is a collection of `Phenotype` instances. This struct is the core collection of individuals
/// being evolved by the `GeneticEngine`. It can be thought of as a Vec of `Phenotype`s and
/// is essentially a light wrapper around such a Vec. The `Population` struct, however, has some
/// additional functionality that allows for sorting and iteration over the individuals in the population.
///
/// Note: Although the `Population` offers mut methods to mut the individuals in the population, the `Population`
/// itself offers no way to increase or decrease the number of individuals in the population. As such, the `Population`
/// should be thought of as an 'immutable' data structure. If you need to add or remove individuals from the population,
/// you should create a new `Population` instance with the new individuals. To further facilitate this way of
/// thinking, the `Population` struct and everything it contains implements the `Clone` trait.
///
/// # Type Parameters
/// - `C`: The type of chromosome used in the genotype, which must implement the `Chromosome` trait.
///
#[derive(Clone, Default)]
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

    pub fn swap(&mut self, a: usize, b: usize) {
        self.is_sorted = false;
        self.individuals.swap(a, b);
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

    pub fn get_scores_ref(&self) -> Vec<&Score> {
        self.individuals
            .iter()
            .filter_map(|i| i.score())
            .collect::<Vec<_>>()
    }
}

impl<C: Chromosome> AsRef<[Phenotype<C>]> for Population<C> {
    fn as_ref(&self) -> &[Phenotype<C>] {
        &self.individuals
    }
}

impl<C: Chromosome> Index<usize> for Population<C> {
    type Output = Phenotype<C>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.individuals[index]
    }
}

impl<C: Chromosome> IndexMut<usize> for Population<C> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.is_sorted = false;
        &mut self.individuals[index]
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
    use crate::objectives::Optimize;
    use crate::{CharChromosome, FloatChromosome};

    #[test]
    fn test_new() {
        let population = Population::<CharChromosome>::new();
        assert_eq!(population.len(), 0);
    }

    #[test]
    fn test_from_vec() {
        let individuals = vec![
            Phenotype::from_chromosomes(vec![CharChromosome::from("hello")], 0),
            Phenotype::from_chromosomes(vec![CharChromosome::from("world")], 0),
        ];

        let population = Population::from_vec(individuals.clone());
        assert_eq!(population.len(), individuals.len());
    }

    #[test]
    fn test_from_fn() {
        let population = Population::from_fn(10, || {
            Phenotype::from_chromosomes(vec![CharChromosome::from("hello")], 0)
        });

        assert_eq!(population.len(), 10);

        for individual in population.iter() {
            assert_eq!(individual.genotype.len(), 1);
            assert_eq!(individual.genotype.iter().next().unwrap().len(), 5);
        }
    }

    #[test]
    fn test_is_empty() {
        let population = Population::<CharChromosome>::new();
        assert!(population.is_empty());
    }

    #[test]
    fn test_sort_by() {
        let mut population = Population::from_fn(10, move || {
            Phenotype::from_chromosomes(vec![FloatChromosome::from(0..10)], 0)
        });

        for i in 0..population.len() {
            population[i].set_score(Some(Score::from_f32(i as f32)));
        }

        let mut minimize_population = population.clone();
        let mut maximize_population = population.clone();

        Optimize::Minimize.sort(&mut minimize_population);
        Optimize::Maximize.sort(&mut maximize_population);

        assert!(minimize_population.is_sorted);
        assert!(maximize_population.is_sorted);

        for i in 0..population.len() {
            assert_eq!(
                minimize_population[i].score().as_ref().unwrap().as_float(),
                i as f32
            );
            assert_eq!(
                maximize_population[i].score().as_ref().unwrap().as_float(),
                (population.len() - i - 1) as f32
            );
        }
    }
}
