use crate::NumericGene;
use crate::{random_provider, Chromosome};

use super::Crossover;

pub struct MeanCrossover {
    pub rate: f32,
}

impl MeanCrossover {
    pub fn new(rate: f32) -> Self {
        MeanCrossover { rate }
    }
}

impl<C: Chromosome> Crossover<C> for MeanCrossover
where
    C::GeneType: NumericGene,
{
    fn cross_rate(&self) -> f32 {
        self.rate
    }

    fn name(&self) -> &'static str {
        "Mean Crossover"
    }

    fn cross_chromosomes(&self, chrom_one: &mut C, chrom_two: &mut C) -> i32 {
        let mut count = 0;

        for (gene_one, gene_two) in chrom_one.iter_mut().zip(chrom_two.iter()) {
            if random_provider::random::<f32>() < self.rate {
                *gene_one = gene_one.mean(gene_two);
                count += 1;
            }
        }

        count
    }
}
