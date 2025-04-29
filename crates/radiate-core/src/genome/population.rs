use super::phenotype::Phenotype;
use crate::cell::MutCell;
use crate::objectives::Scored;
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
    pub individuals: Vec<MutCell<Phenotype<C>>>,
    pub is_sorted: bool,
}

impl<C: Chromosome> Population<C> {
    /// Create a new instance of the Population with the given individuals.
    /// This will set the is_sorted flag to false.
    pub fn new(individuals: Vec<Phenotype<C>>) -> Self {
        Population {
            individuals: individuals.into_iter().map(MutCell::from).collect(),
            is_sorted: false,
        }
    }

    pub fn get(&self, index: usize) -> Option<&Phenotype<C>> {
        self.individuals.get(index).map(|cell| cell.get())
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Phenotype<C>> {
        self.is_sorted = false;
        self.individuals.get_mut(index).map(|cell| cell.get_mut())
    }

    pub fn get_cell_mut(&mut self, index: usize) -> Option<&mut MutCell<Phenotype<C>>> {
        self.is_sorted = false;
        self.individuals.get_mut(index)
    }

    pub fn get_cell(&self, index: usize) -> Option<&MutCell<Phenotype<C>>> {
        self.individuals.get(index)
    }

    pub fn push(&mut self, individual: impl Into<MutCell<Phenotype<C>>>) {
        self.is_sorted = false;
        self.individuals.push(individual.into());
    }

    pub fn iter(&self) -> impl Iterator<Item = &Phenotype<C>> {
        MemberIter {
            population: self,
            index: 0,
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Phenotype<C>> {
        self.is_sorted = false;
        MemberIterMut {
            population: self,
            index: 0,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.individuals.len()
    }

    pub fn clear(&mut self) {
        self.is_sorted = false;
        self.individuals.clear();
    }

    pub fn get_scores(&self) -> Vec<&Score> {
        self.individuals
            .iter()
            .filter_map(|individual| individual.score())
            .collect()
    }

    /// Sort the individuals in the population using the given closure.
    /// This will set the is_sorted flag to true.
    pub fn sort_by<F>(&mut self, mut f: F)
    where
        F: FnMut(&Phenotype<C>, &Phenotype<C>) -> std::cmp::Ordering,
    {
        if self.is_sorted {
            return;
        }

        let new_sort_fn =
            move |a: &MutCell<Phenotype<C>>, b: &MutCell<Phenotype<C>>| f(a.get(), b.get());

        self.individuals.sort_by(new_sort_fn);
        self.is_sorted = true;
    }

    pub fn is_empty(&self) -> bool {
        self.individuals.is_empty()
    }

    pub fn get_pair_mut(
        &mut self,
        first: usize,
        second: usize,
    ) -> (&mut Phenotype<C>, &mut Phenotype<C>) {
        if first < second {
            let (left, right) = self.individuals.split_at_mut(second);
            (left[first].get_mut(), right[0].get_mut())
        } else {
            let (left, right) = self.individuals.split_at_mut(first);
            (right[0].get_mut(), left[second].get_mut())
        }
    }
}

impl<C: Chromosome> From<Vec<Phenotype<C>>> for Population<C> {
    fn from(individuals: Vec<Phenotype<C>>) -> Self {
        Population {
            individuals: individuals.into_iter().map(MutCell::from).collect(),
            is_sorted: false,
        }
    }
}

impl<C: Chromosome> From<Vec<MutCell<Phenotype<C>>>> for Population<C> {
    fn from(individuals: Vec<MutCell<Phenotype<C>>>) -> Self {
        Population {
            individuals,
            is_sorted: false,
        }
    }
}

impl<C: Chromosome> AsRef<[MutCell<Phenotype<C>>]> for Population<C> {
    fn as_ref(&self) -> &[MutCell<Phenotype<C>>] {
        self.individuals.as_ref()
    }
}

impl<C: Chromosome> AsMut<[MutCell<Phenotype<C>>]> for Population<C> {
    fn as_mut(&mut self) -> &mut [MutCell<Phenotype<C>>] {
        self.is_sorted = false;
        self.individuals.as_mut()
    }
}

impl<C: Chromosome> Index<Range<usize>> for Population<C> {
    type Output = [MutCell<Phenotype<C>>];
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
        self.is_sorted = false;
        self.individuals[index].get_mut()
    }
}

impl<C: Chromosome> IntoIterator for Population<C> {
    type Item = Phenotype<C>;
    type IntoIter = std::vec::IntoIter<Phenotype<C>>;

    fn into_iter(self) -> Self::IntoIter {
        self.individuals
            .into_iter()
            .map(|cell| cell.into_inner())
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl<C: Chromosome> FromIterator<Phenotype<C>> for Population<C> {
    fn from_iter<I: IntoIterator<Item = Phenotype<C>>>(iter: I) -> Self {
        let individuals = iter
            .into_iter()
            .map(MutCell::from)
            .collect::<Vec<MutCell<Phenotype<C>>>>();
        Population {
            individuals,
            is_sorted: false,
        }
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

        Population {
            individuals: individuals.into_iter().map(MutCell::from).collect(),
            is_sorted: false,
        }
    }
}

impl<C: Chromosome + Debug> Debug for Population<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Population [\n")?;
        for individual in &self.individuals {
            write!(f, "{:?},\n ", individual.get())?;
        }
        write!(f, "]")
    }
}

impl<C: Chromosome> Scored for MutCell<Phenotype<C>> {
    fn score(&self) -> Option<&Score> {
        self.get().score()
    }
}

struct MemberIter<'a, C: Chromosome> {
    population: &'a Population<C>,
    index: usize,
}

impl<'a, C: Chromosome> Iterator for MemberIter<'a, C> {
    type Item = &'a Phenotype<C>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.population.len() {
            let member = &self.population.individuals[self.index];
            self.index += 1;
            Some(member.get())
        } else {
            None
        }
    }
}

struct MemberIterMut<'a, C: Chromosome> {
    population: *mut Population<C>,
    index: usize,
    _marker: std::marker::PhantomData<&'a mut Population<C>>,
}
impl<'a, C: Chromosome> Iterator for MemberIterMut<'a, C> {
    type Item = &'a mut Phenotype<C>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let population = &mut *self.population;
            if self.index < population.len() {
                let member = &mut population.individuals[self.index];
                self.index += 1;
                Some(member.get_mut())
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Score, char::CharChromosome, float::FloatChromosome, objectives::Optimize};

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
            population[i].set_score(Some(Score::from_usize(i)));
        }

        let mut minimize_population = population.clone();
        let mut maximize_population = population.clone();

        Optimize::Minimize.sort(&mut minimize_population);
        Optimize::Maximize.sort(&mut maximize_population);

        // assert!(minimize_population.is_sorted);
        // assert!(maximize_population.is_sorted);

        for i in 0..population.len() {
            assert_eq!(minimize_population[i].score().unwrap().as_usize(), i);
            assert_eq!(
                maximize_population[i].score().unwrap().as_usize(),
                population.len() - i - 1
            );
        }
    }
}
