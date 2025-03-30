pub mod chromosomes;
pub mod genotype;
pub mod phenotype;
pub mod population;
pub mod species;

pub use chromosomes::*;
pub use genotype::*;
pub use phenotype::*;
pub use population::*;
pub use species::*;

pub trait PopulationView<C: Chromosome>: AsRef<[Phenotype<C>]> + AsMut<[Phenotype<C>]> {
    fn push(&mut self, individual: Phenotype<C>);

    fn clear(&mut self);

    fn swap(&mut self, a: usize, b: usize) {
        let len = self.len();
        if a < len && b < len {
            self.as_mut().swap(a, b);
        } else {
            panic!("Index out of bounds: a={}, b={}, len={}", a, b, len);
        }
    }

    fn len(&self) -> usize {
        self.as_ref().len()
    }

    fn get(&self, index: usize) -> &Phenotype<C> {
        &self.as_ref()[index]
    }

    fn get_mut(&mut self, index: usize) -> &mut Phenotype<C> {
        &mut self.as_mut()[index]
    }

    fn iter(&self) -> std::slice::Iter<Phenotype<C>> {
        self.as_ref().iter()
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<Phenotype<C>> {
        self.as_mut().iter_mut()
    }

    fn sort_by<F>(&mut self, f: F)
    where
        F: FnMut(&Phenotype<C>, &Phenotype<C>) -> std::cmp::Ordering,
    {
        self.as_mut().sort_by(f);
    }
}
