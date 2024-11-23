use super::{genes::gene::Gene, phenotype::Phenotype};

/// A ```Population``` is a collection of ```Phenotype``` instances. This struct is the core collection of individuals 
/// being evolved by the ```GeneticEngine```. It can be thought of as a Vec of ```Phenotype```s and 
/// is essentially a light wrapper around such a Vec. The ```Population``` struct, however, has some
/// additional functionality that allows for sorting and iteration over the individuals in the population.
/// 
/// Note: Although the ```Population``` offers mut methods to mut the individuals in the population, the ```Population```
/// itself offers no way to increase or decrease the number of individuals in the population. As such, the ```Population```
/// should be thought of as an 'immutable' data structure. If you need to add or remove individuals from the population,
/// you should create a new ```Population``` instance with the new individuals. To further facilitate this way of 
/// thinking, the ```Population``` struct and everyhing it contains implements the ```Clone``` trait. 
/// 
/// # Type Parameters
/// - `G`: The type of gene used in the genetic algorithm, which must implement the `Gene` trait.
/// - `A`: The type of the allele associated with the gene - the gene's "expression".
pub struct Population<G, A>
where
    G: Gene<G, A>,
{
    pub individuals: Vec<Phenotype<G, A>>,
    pub is_sorted: bool,
}

impl<G, A> Population<G, A>
where
    G: Gene<G, A>,
{
    /// Create a new instance of the Population. This will create a new instance with an 
    /// empty list of individuals and the is_sorted flag set to false.
    pub fn new() -> Self {
        Population {
            individuals: Vec::new(),
            is_sorted: false,
        }
    }

    pub fn get(&self, index: usize) -> &Phenotype<G, A> {
        self.individuals.get(index).expect("Index out of bounds")
    }

    /// Get a mutable reference to the individual at the given index. This will set the is_sorted flag to false
    /// because we cannot guarantee that the individual's ```Score``` (fitness) has not changed.
    pub fn get_mut(&mut self, index: usize) -> &mut Phenotype<G, A> {
        self.is_sorted = false;
        self.individuals
            .get_mut(index)
            .expect("Index out of bounds")
    }

    /// Set the individual at the given index. This will set the is_sorted flag to false
    /// because we cannot guarantee that the individual is in the correct order.
    pub fn set(&mut self, index: usize, individual: Phenotype<G, A>) {
        self.individuals[index] = individual;
        self.is_sorted = false;
    }

    pub fn iter(&self) -> std::slice::Iter<Phenotype<G, A>> {
        self.individuals.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<Phenotype<G, A>> {
        self.is_sorted = false;
        self.individuals.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.individuals.len()
    }

    /// Sort the individuals in the population using the given closure. This will set the is_sorted flag to true.
    pub fn sort_by<F>(&mut self, f: F)
    where
        F: FnMut(&Phenotype<G, A>, &Phenotype<G, A>) -> std::cmp::Ordering,
    {
        if self.is_sorted {
            return;
        }

        self.individuals.sort_by(f);
        self.is_sorted = true;
    }

    pub fn from_vec(individuals: Vec<Phenotype<G, A>>) -> Self {
        Population {
            individuals,
            is_sorted: false,
        }
    }

    pub fn from_fn<F>(size: usize, f: F) -> Self
    where
        F: Fn() -> Phenotype<G, A>,
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
}

impl<G, A> IntoIterator for Population<G, A>
where
    G: Gene<G, A>,
{
    type Item = Phenotype<G, A>;
    type IntoIter = std::vec::IntoIter<Phenotype<G, A>>;

    fn into_iter(self) -> Self::IntoIter {
        self.individuals.into_iter()
    }
}

impl<G, A> FromIterator<Phenotype<G, A>> for Population<G, A>
where
    G: Gene<G, A>,
{
    fn from_iter<I: IntoIterator<Item = Phenotype<G, A>>>(iter: I) -> Self {
        let individuals = iter.into_iter().collect();
        Population {
            individuals,
            is_sorted: false,
        }
    }
}

impl<G, A> Clone for Population<G, A>
where
    G: Gene<G, A>,
{
    fn clone(&self) -> Self {
        Population {
            individuals: self.individuals.clone(),
            is_sorted: self.is_sorted,
        }
    }
}

impl<G, A> std::fmt::Debug for Population<G, A>
where
    G: Gene<G, A> + std::fmt::Debug,
{
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
    use crate::engines::genome::genes::char_gene::CharGene;

    #[test]
    fn test_new() {
        let population = Population::<CharGene, char>::new();
        assert_eq!(population.len(), 0);
    }
}
