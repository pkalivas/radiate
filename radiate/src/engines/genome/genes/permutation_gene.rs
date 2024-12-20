use std::{fmt::Debug, sync::Arc};

use super::{Gene, Valid};

/// The `PermutationGene` is a gene that represents a permutation of a set of alleles. The gene has an index
/// that represents the position of the allele in the alleles vector. The alleles vector is a set of unique
/// values. The gene is valid if the index is less than the length of the alleles vector. This gene is useful
/// for representing permutations of values, such as the order of cities in a TSP problem.
///
/// # Type Parameters
/// - `A`: The type of the alleles.
///
#[derive(Debug, Clone, PartialEq)]
pub struct PermutationGene<A: PartialEq + Clone> {
    pub index: usize,
    pub alleles: Arc<Vec<A>>,
}

impl<A: PartialEq + Clone> PermutationGene<A> {
    pub fn new(index: usize, alleles: Arc<Vec<A>>) -> Self {
        PermutationGene { index, alleles }
    }
}

impl<A: PartialEq + Clone> Gene for PermutationGene<A> {
    type Allele = A;

    fn allele(&self) -> &Self::Allele {
        &self.alleles[self.index]
    }

    fn new_instance(&self) -> Self {
        PermutationGene {
            index: self.index,
            alleles: Arc::clone(&self.alleles),
        }
    }

    fn with_allele(&self, allele: &Self::Allele) -> Self {
        // Find the index of the allele in the alleles vector - this implies that `self.alleles`
        // is a set of unique values.
        let index = self.alleles.iter().position(|x| x == allele).unwrap();
        PermutationGene {
            index,
            alleles: Arc::clone(&self.alleles),
        }
    }
}

impl<A: PartialEq + Clone> Valid for PermutationGene<A> {
    fn is_valid(&self) -> bool {
        self.index < self.alleles.len()
    }
}
