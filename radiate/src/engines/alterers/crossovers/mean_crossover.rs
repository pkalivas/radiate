use crate::Chromosome;
use crate::NumericGene;
use crate::RandomProvider;

use super::Crossover;

pub struct MeanCrossover {
    pub rate: f32,
}

impl MeanCrossover {
    pub fn new(rate: f32) -> Self {
        MeanCrossover { rate }
    }
}

impl<G, A> Crossover<G, A> for MeanCrossover
where
    G: NumericGene<G, A>,
{
    fn cross_rate(&self) -> f32 {
        self.rate
    }

    fn name(&self) -> &'static str {
        "Mean Crossover"
    }

    fn cross_chromosomes(
        &self,
        chrom_one: &mut Chromosome<G, A>,
        chrom_two: &mut Chromosome<G, A>,
    ) -> i32 {
        let mut count = 0;

        for (gene_one, gene_two) in chrom_one.iter_mut().zip(chrom_two.iter()) {
            if RandomProvider::random::<f32>() < self.rate {
                *gene_one = gene_one.mean(gene_two);
                count += 1;
            }
        }

        count
    }
}
