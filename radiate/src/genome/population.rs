use super::phenotype::Phenotype;
use crate::{Chromosome, Score};
use std::fmt::Debug;
use std::ops::{Index, IndexMut, Range};

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

#[derive(Clone, Default)]
pub struct Population<C: Chromosome> {
    pub individuals: Vec<Phenotype<C>>,
}

impl<C: Chromosome> Population<C> {
    /// Create a new instance of the Population with the given individuals.
    /// This will set the is_sorted flag to false.
    pub fn new(individuals: Vec<Phenotype<C>>) -> Self {
        Population { individuals }
    }

    pub fn push(&mut self, individual: Phenotype<C>) {
        self.individuals.push(individual);
    }

    pub fn get(&self, index: usize) -> &Phenotype<C> {
        &self.individuals[index]
    }

    pub fn get_mut(&mut self, index: usize) -> &mut Phenotype<C> {
        &mut self.individuals[index]
    }

    pub fn iter(&self) -> std::slice::Iter<Phenotype<C>> {
        self.individuals.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<Phenotype<C>> {
        self.individuals.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.individuals.len()
    }

    pub fn clear(&mut self) {
        self.individuals.clear();
    }

    /// Swap the individuals at the given indices. This will set the is_sorted flag to false
    /// because the order of the individuals has changed and we don't know if the order
    /// has changed to benefit the order or not. Therefore, don't use this method to
    /// sort the population, use the `sort_by` method instead.
    pub fn swap(&mut self, a: usize, b: usize) {
        self.individuals.swap(a, b);
    }

    pub fn get_scores(&self) -> Vec<Score> {
        self.individuals
            .iter()
            .filter_map(|individual| individual.score())
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.individuals.is_empty()
    }

    pub fn get_pair_mut(
        &mut self,
        first: usize,
        second: usize,
    ) -> (&mut Phenotype<C>, &mut Phenotype<C>) {
        let (one, two) = if first < second {
            let (left, right) = self.individuals.split_at_mut(second);
            (&mut left[first], &mut right[0])
        } else {
            let (left, right) = self.individuals.split_at_mut(first);
            (&mut right[0], &mut left[second])
        };

        (one, two)
    }

    pub fn individuals_in_generation(&self, generation: usize) -> Vec<Phenotype<C>> {
        self.individuals
            .iter()
            .filter(|individual| individual.generation() == generation)
            .map(|member| Phenotype::clone(member))
            .collect()
    }
}

impl<C: Chromosome> AsRef<[Phenotype<C>]> for Population<C> {
    fn as_ref(&self) -> &[Phenotype<C>] {
        &self.individuals
    }
}

impl<C: Chromosome> AsMut<[Phenotype<C>]> for Population<C> {
    fn as_mut(&mut self) -> &mut [Phenotype<C>] {
        &mut self.individuals
    }
}

impl<C: Chromosome> Index<Range<usize>> for Population<C> {
    type Output = [Phenotype<C>];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.individuals[index]
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
        Population { individuals }
    }
}

/// Create a new instance of the Population from the given size and closure.
/// This will iterate the given closure `size` times and collect
/// the results into a Vec of new individuals.
impl<C: Chromosome, F> From<(usize, F)> for Population<C>
where
    F: Fn() -> Phenotype<C>,
{
    fn from((size, f): (usize, F)) -> Self {
        let mut individuals = Vec::with_capacity(size);
        for _ in 0..size {
            individuals.push(f());
        }

        Population { individuals }
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
    use crate::{
        Score, Scored, char::CharChromosome, float::FloatChromosome, objectives::Optimize,
    };

    #[test]
    fn test_new() {
        let population = Population::<CharChromosome>::default();
        assert_eq!(population.len(), 0);
    }

    #[test]
    fn test_from_vec() {
        let individuals = vec![
            Phenotype::from((vec![CharChromosome::from("hello")], 0)),
            Phenotype::from((vec![CharChromosome::from("world")], 0)),
        ];

        let population = Population::new(individuals.clone());
        assert_eq!(population.len(), individuals.len());
    }

    #[test]
    fn test_from_fn() {
        let population = Population::from((10, || {
            Phenotype::from((vec![CharChromosome::from("hello")], 0))
        }));

        assert_eq!(population.len(), 10);

        for individual in population.iter() {
            assert_eq!(individual.genotype().len(), 1);
            assert_eq!(individual.genotype().iter().next().unwrap().len(), 5);
        }
    }

    #[test]
    fn test_is_empty() {
        let population = Population::<CharChromosome>::default();
        assert!(population.is_empty());
    }

    #[test]
    fn test_sort_by() {
        let mut population = Population::from((10, || {
            Phenotype::from((vec![FloatChromosome::from((10, -10.0..10.0))], 0))
        }));

        for i in 0..population.len() {
            population[i].set_score(Some(Score::from(i)));
        }

        let mut minimize_population = population.clone();
        let mut maximize_population = population.clone();

        Optimize::Minimize.sort(&mut minimize_population);
        Optimize::Maximize.sort(&mut maximize_population);

        for i in 0..population.len() {
            assert_eq!(minimize_population[i].score().as_usize(), i);
            assert_eq!(
                maximize_population[i].score().as_usize(),
                population.len() - i - 1
            );
        }
    }
}
