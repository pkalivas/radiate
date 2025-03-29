use super::{Chromosome, Gene, Valid};
use std::{fmt::Debug, sync::Arc};

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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PermutationChromosome<A: PartialEq + Clone> {
    pub genes: Vec<PermutationGene<A>>,
    pub alleles: Arc<Vec<A>>,
}

impl<A: PartialEq + Clone> PermutationChromosome<A> {
    pub fn new(genes: Vec<PermutationGene<A>>, alleles: Arc<Vec<A>>) -> Self {
        PermutationChromosome { genes, alleles }
    }
}

impl<A: PartialEq + Clone> Chromosome for PermutationChromosome<A> {
    type Gene = PermutationGene<A>;
}

impl<A: PartialEq + Clone> Valid for PermutationChromosome<A> {
    fn is_valid(&self) -> bool {
        // Check if the genes are a valid permutation of the alleles
        let mut bit_set = vec![false; self.alleles.len()];
        self.genes.iter().all(|gene| {
            let index = gene.index;
            if bit_set[index] {
                return false;
            }
            bit_set[index] = true;
            true
        })
    }
}

impl<A: PartialEq + Clone> AsRef<[PermutationGene<A>]> for PermutationChromosome<A> {
    fn as_ref(&self) -> &[PermutationGene<A>] {
        &self.genes
    }
}

impl<A: PartialEq + Clone> AsMut<[PermutationGene<A>]> for PermutationChromosome<A> {
    fn as_mut(&mut self) -> &mut [PermutationGene<A>] {
        &mut self.genes
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_permutation_gene() {
        let alleles = Arc::new(vec![1, 2, 3, 4]);
        let gene = PermutationGene::new(0, Arc::clone(&alleles));

        assert_eq!(gene.allele(), &1);
        assert!(gene.is_valid());
    }

    #[test]
    fn test_permutation_chromosome() {
        let alleles = Arc::new(vec![1, 2, 3, 4]);
        let genes = vec![
            PermutationGene::new(0, Arc::clone(&alleles)),
            PermutationGene::new(1, Arc::clone(&alleles)),
            PermutationGene::new(2, Arc::clone(&alleles)),
            PermutationGene::new(3, Arc::clone(&alleles)),
        ];
        let chromosome = PermutationChromosome::new(genes.clone(), Arc::clone(&alleles));

        assert_eq!(chromosome.genes.len(), 4);
        assert!(chromosome.is_valid());
        for (i, gene) in chromosome.genes.iter().enumerate() {
            assert_eq!(gene.index, i);
            assert_eq!(gene.allele(), &alleles[i]);
        }
    }
}
