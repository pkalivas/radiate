use crate::alter::AlterType;
use crate::{random_provider, Chromosome};
use crate::{Alter, NumericGene};

pub struct MeanCrossover {
    pub rate: f32,
}

impl MeanCrossover {
    pub fn new(rate: f32) -> Self {
        MeanCrossover { rate }
    }
}

impl<C: Chromosome> Alter<C> for MeanCrossover
where
    C::GeneType: NumericGene,
{
    fn name(&self) -> &'static str {
        "Mean Crossover"
    }

    fn rate(&self) -> f32 {
        self.rate
    }

    fn alter_type(&self) -> AlterType {
        AlterType::Crossover
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
