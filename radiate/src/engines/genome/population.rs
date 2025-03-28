use super::phenotype::Phenotype;
use crate::Chromosome;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::sync::{Arc, RwLockReadGuard};
use std::sync::{RwLock, RwLockWriteGuard};

#[derive(Debug)]
pub struct SyncCell<T> {
    inner: Arc<RwLock<T>>,
}

impl<T> SyncCell<T> {
    pub fn new(value: T) -> Self {
        SyncCell {
            inner: Arc::new(RwLock::new(value)),
        }
    }

    pub fn clone(other: &SyncCell<T>) -> Self {
        // This will create a new MemberCell with the same inner Phenotype.
        // Note: This will not clone the inner Phenotype itself, but rather create a new reference to it.
        SyncCell {
            inner: Arc::clone(&other.inner),
        }
    }

    pub fn into_inner(self) -> T {
        // This will consume the MemberCell and return the inner Phenotype.
        // Note: This will not drop the RwLock, so be cautious when using this.
        Arc::try_unwrap(self.inner)
            .ok()
            .expect("Multiple references to SyncCell exist")
            .into_inner()
            .expect("RwLock poisoned")
    }

    pub fn read(&self) -> SyncCellGuard<T> {
        let read_lock = self.inner.read().unwrap();
        SyncCellGuard { inner: read_lock }
    }

    pub fn write(&self) -> SyncCellGuardMut<T> {
        let write_lock = self.inner.write().unwrap();
        SyncCellGuardMut { inner: write_lock }
    }

    pub fn set(&self, value: T) {
        let mut write_lock = self.inner.write().unwrap();
        *write_lock = value;
    }
}

impl<T: Clone> Clone for SyncCell<T> {
    fn clone(&self) -> Self {
        let inner = self.inner.read().unwrap().clone();
        SyncCell {
            inner: Arc::new(RwLock::new(inner)),
        }
    }
}

impl<C: Chromosome> From<Phenotype<C>> for SyncCell<Phenotype<C>> {
    fn from(individual: Phenotype<C>) -> Self {
        SyncCell::new(individual)
    }
}

pub struct SyncCellGuard<'a, T> {
    inner: RwLockReadGuard<'a, T>,
}

impl<T> Deref for SyncCellGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct SyncCellGuardMut<'a, T> {
    inner: RwLockWriteGuard<'a, T>,
}

impl<T> Deref for SyncCellGuardMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for SyncCellGuardMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

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
    pub individuals: Vec<SyncCell<Phenotype<C>>>,
    pub is_sorted: bool,
}

impl<C: Chromosome> Population<C> {
    /// Create a new instance of the Population with the given individuals.
    /// This will set the is_sorted flag to false.
    pub fn new<M: Into<SyncCell<Phenotype<C>>>>(individuals: Vec<M>) -> Self {
        Population {
            individuals: individuals
                .into_iter()
                .map(|individual| individual.into())
                .collect(),
            is_sorted: false,
        }
    }

    pub fn get(&self, index: usize) -> Option<&SyncCell<Phenotype<C>>> {
        self.individuals.get(index)
    }

    pub fn iter(&self) -> impl Iterator<Item = &SyncCell<Phenotype<C>>> {
        self.individuals.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut SyncCell<Phenotype<C>>> {
        self.is_sorted = false;
        self.individuals.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.individuals.len()
    }

    /// Swap the individuals at the given indices. This will set the is_sorted flag to false
    /// because the order of the individuals has changed and we don't know if the order
    /// has changed to benefit the order or not. Therefore, don't use this method to
    /// sort the population, use the `sort_by` method instead.
    pub fn swap(&mut self, a: usize, b: usize) {
        self.is_sorted = false;
        self.individuals.swap(a, b);
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

        self.individuals.sort_by(|a, b| {
            let a_guard = a.read();
            let b_guard = b.read();
            // We need to unwrap the Phenotype from the SyncCellGuard to compare them.
            let a_phenotype = a_guard.deref(); // Deref to get the inner Phenotype
            let b_phenotype = b_guard.deref(); // Deref to get the inner Phenotype
            // Call the provided closure to compare the two Phenotypes.
            f(a_phenotype, b_phenotype)
        });
        self.is_sorted = true;
    }

    pub fn is_empty(&self) -> bool {
        self.individuals.is_empty()
    }

    pub fn get_scores_ref(&self) -> Vec<&[f32]> {
        Vec::new()
        // self.individuals
        //     .iter()
        //     .filter_map(|i| i.score())
        //     .map(|s| s.as_ref())
        //     .collect::<Vec<_>>()
    }

    pub fn filter_drain<F: Fn(&Phenotype<C>) -> bool>(&mut self, filter: F) -> Self {
        let mut new_population = Vec::new();
        let mut old_population = Vec::new();

        for individual in self.individuals.drain(..) {
            if filter(&individual.read()) {
                new_population.push(individual);
            } else {
                old_population.push(individual);
            }
        }

        self.individuals = old_population;
        Population::new(new_population)
    }
}

impl<C: Chromosome> Index<usize> for Population<C> {
    type Output = SyncCell<Phenotype<C>>;

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
    type Item = SyncCell<Phenotype<C>>;
    type IntoIter = std::vec::IntoIter<SyncCell<Phenotype<C>>>;
    fn into_iter(self) -> Self::IntoIter {
        self.individuals.into_iter()
    }
}

impl<C: Chromosome, T: Into<SyncCell<Phenotype<C>>>> FromIterator<T> for Population<C> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let individuals = iter
            .into_iter()
            .map(|individual| individual.into())
            .collect::<Vec<SyncCell<Phenotype<C>>>>();
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
            individuals.push(SyncCell::new(f()));
        }

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
            assert_eq!(individual.read().genotype().len(), 1);
            assert_eq!(individual.read().genotype().iter().next().unwrap().len(), 5);
        }
    }

    #[test]
    fn test_is_empty() {
        let population = Population::<CharChromosome>::default();
        assert!(population.is_empty());
    }

    #[test]
    fn test_sort_by() {
        let population = Population::from((10, || {
            Phenotype::from((vec![FloatChromosome::from((10, -10.0..10.0))], 0))
        }));

        for i in 0..population.len() {
            population
                .get(i)
                .unwrap()
                .write()
                .set_score(Some(Score::from_usize(i)));
        }

        let mut minimize_population = population.clone();
        let mut maximize_population = population.clone();

        Optimize::Minimize.sort(&mut minimize_population);
        Optimize::Maximize.sort(&mut maximize_population);

        assert!(minimize_population.is_sorted);
        assert!(maximize_population.is_sorted);

        for i in 0..population.len() {
            assert_eq!(
                minimize_population
                    .get(i)
                    .unwrap()
                    .read()
                    .score()
                    .unwrap()
                    .as_usize(),
                i
            );
            assert_eq!(
                maximize_population
                    .get(i)
                    .unwrap()
                    .read()
                    .score()
                    .unwrap()
                    .as_usize(),
                population.len() - i - 1
            );
        }
    }
}
