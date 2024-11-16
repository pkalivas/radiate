use crate::engines::genome::chromosome::Chromosome;
use crate::engines::genome::genes::float_gene::FloatGene;
use crate::engines::genome::genes::gene::{BoundGene, Gene};
use crate::engines::genome::genotype::Genotype;

use super::Codex;

pub struct FloatCodex {
    pub num_chromosomes: usize,
    pub num_genes: usize,
    pub min: f32,
    pub max: f32,
    pub lower_bound: f32,
    pub upper_bound: f32,
}

impl FloatCodex {
    pub fn new(num_chromosomes: usize, num_genes: usize, min: f32, max: f32) -> Self {
        FloatCodex {
            num_chromosomes,
            num_genes,
            min,
            max,
            lower_bound: f32::MIN,
            upper_bound: f32::MAX,
        }
    }

    pub fn with_bounds(mut self, lower_bound: f32, upper_bound: f32) -> Self {
        self.lower_bound = lower_bound;
        self.upper_bound = upper_bound;
        self
    }

    pub fn scalar(min: f32, max: f32) -> Self {
        FloatCodex::new(1, 1, min, max)
    }
}

impl Codex<FloatGene, f32, Vec<Vec<f32>>> for FloatCodex {
    fn encode(&self) -> Genotype<FloatGene, f32> {
        Genotype {
            chromosomes: (0..self.num_chromosomes)
                .into_iter()
                .map(|_| {
                    Chromosome::from_genes(
                        (0..self.num_genes)
                            .into_iter()
                            .map(|_| {
                                FloatGene::new(self.min, self.max)
                                    .with_bounds(self.lower_bound, self.upper_bound)
                            })
                            .collect::<Vec<FloatGene>>(),
                    )
                })
                .collect::<Vec<Chromosome<FloatGene, f32>>>(),
        }
    }

    fn decode(&self, genotype: &Genotype<FloatGene, f32>) -> Vec<Vec<f32>> {
        genotype
            .iter()
            .map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<f32>>()
            })
            .collect::<Vec<Vec<f32>>>()
    }
}
