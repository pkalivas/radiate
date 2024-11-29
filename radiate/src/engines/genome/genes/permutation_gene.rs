use std::{fmt::Debug, sync::Arc};

use super::{Gene, Valid};

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
