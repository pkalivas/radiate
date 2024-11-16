use crate::engines::genome::{
    chromosome::Chromosome, genes::bit_gene::BitGene, genes::gene::Gene, genotype::Genotype,
};

use super::Codex;

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
