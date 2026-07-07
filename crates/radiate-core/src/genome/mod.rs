pub mod chromosomes;
pub mod ecosystem;
pub mod genotype;
pub mod phenotype;
pub mod population;
pub mod species;

pub use chromosomes::{
    BitChromosome, BitGene, BoundedGene, CharChromosome, CharGene, Chromosome, FloatChromosome,
    FloatGene, Gene, IntChromosome, IntGene, PermutationChromosome, PermutationGene, Valid,
};
pub use ecosystem::Ecosystem;
pub use genotype::Genotype;
pub use phenotype::Phenotype;
pub use population::Population;
pub use species::{Species, SpeciesId};

pub trait GetPairMut<T> {
    fn get_pair_mut(&mut self, index1: usize, index2: usize) -> Option<(&mut T, &mut T)>;
}

impl<T> GetPairMut<T> for Vec<T> {
    fn get_pair_mut(&mut self, first: usize, second: usize) -> Option<(&mut T, &mut T)> {
        get_pair_mut_internal(self.as_mut_slice(), first, second)
    }
}

impl<T> GetPairMut<T> for &mut [T] {
    fn get_pair_mut(&mut self, first: usize, second: usize) -> Option<(&mut T, &mut T)> {
        get_pair_mut_internal(self, first, second)
    }
}

#[inline]
fn get_pair_mut_internal<T>(
    slice: &mut [T],
    first: usize,
    second: usize,
) -> Option<(&mut T, &mut T)> {
    if first == second {
        None
    } else if first < second {
        let (left, right) = slice.split_at_mut(second);
        Some((&mut left[first], &mut right[0]))
    } else {
        let (left, right) = slice.split_at_mut(first);
        Some((&mut right[0], &mut left[second]))
    }
}
