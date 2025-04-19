use super::Codex;
use crate::{BitChromosome, BitGene, Chromosome, Gene, Genotype};
use std::sync::Arc;

/// A `Codex` for a subset of items. This is useful for problems where the goal is to find the best subset of items
/// from a larger set of items. The `encode` function creates a `Genotype` with a single chromosome of `BitGenes`
/// where each gene represents an item in the `items` vector. The `decode` function creates a `Vec<&T>` from the
/// `Genotype` where the `Vec` contains the items that are selected by the `BitGenes` - the `true` genes.
///
/// A `SubSetCodex` is useful for problems like the Knapsack problem, where the goal is to find the best subset of items
/// that fit in a knapsack. The `items` vector would contain the items that can be placed in the knapsack and the `Genotype`
/// would contain `BitGenes` that represent weather or not the item is in the knapsack.
#[derive(Clone)]
pub struct SubSetCodex<T> {
    items: Arc<[Arc<T>]>,
}

impl<T> SubSetCodex<T> {
    pub fn new(items: Vec<T>) -> Self {
        SubSetCodex {
            items: items.into_iter().map(Arc::new).collect(),
        }
    }
}

impl<T> Codex<BitChromosome, Vec<Arc<T>>> for SubSetCodex<T> {
    fn encode(&self) -> Genotype<BitChromosome> {
        Genotype::new(vec![BitChromosome {
            genes: self
                .items
                .iter()
                .map(|_| BitGene::new())
                .collect::<Vec<BitGene>>(),
        }])
    }

    fn decode(&self, genotype: &Genotype<BitChromosome>) -> Vec<Arc<T>> {
        let mut result = Vec::new();
        for (i, gene) in genotype.iter().next().unwrap().iter().enumerate() {
            if *gene.allele() {
                result.push(Arc::clone(&self.items[i]));
            }
        }

        result
    }
}
