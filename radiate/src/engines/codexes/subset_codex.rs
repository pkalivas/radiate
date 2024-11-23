use crate::engines::genome::{
    chromosome::Chromosome, genes::bit_gene::BitGene, genes::gene::Gene, genotype::Genotype,
};

use super::Codex;

/// A `Codex` for a subset of items. This is useful for problems where the goal is to find the best subset of items
/// from a larger set of items. The `encode` function creates a `Genotype` with a single chromosome of `BitGenes`
/// where each gene represents an item in the `items` vector. The `decode` function creates a `Vec<&T>` from the
/// `Genotype` where the `Vec` contains the items that are selected by the `BitGenes` - the `true` genes.
///
/// A `SubSetCodex` is useful for problems like the Knapsack problem, where the goal is to find the best subset of items
/// that fit in a knapsack. The `items` vector would contain the items that can be placed in the knapsack and the `Genotype`
/// would contain `BitGenes` that represent whether or not the item is in the knapsack.
pub struct SubSetCodex<'a, T> {
    pub items: &'a Vec<T>,
}

impl<'a, T> SubSetCodex<'a, T> {
    pub fn new(items: &'a Vec<T>) -> Self {
        Self { items }
    }
}

impl<'a, T> Codex<BitGene, bool, Vec<&'a T>> for SubSetCodex<'a, T> {
    fn encode(&self) -> Genotype<BitGene, bool> {
        Genotype {
            chromosomes: vec![Chromosome::from_genes(
                self.items
                    .iter()
                    .map(|_| BitGene::new())
                    .collect::<Vec<BitGene>>(),
            )],
        }
    }

    fn decode(&self, genotype: &Genotype<BitGene, bool>) -> Vec<&'a T> {
        let mut result = Vec::new();
        for (i, gene) in genotype.iter().next().unwrap().iter().enumerate() {
            if *gene.allele() {
                result.push(&self.items[i]);
            }
        }

        result
    }
}
