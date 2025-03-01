use super::{AlterAction, Alterer, Crossover, IntoAlter};
use crate::{Chromosome, FloatGene, Gene, random_provider};

pub struct BlendCrossover {
    rate: f32,
    alpha: f32,
}

impl BlendCrossover {
    /// Create a new instance of the `BlendCrossover` with the given rate and alpha.
    /// The rate must be between 0.0 and 1.0, and the alpha must be between 0.0 and 1.0.
    pub fn new(rate: f32, alpha: f32) -> Self {
        if rate < 0.0 || rate > 1.0 {
            panic!("Rate must be between 0 and 1");
        }

        if alpha < 0.0 || alpha > 1.0 {
            panic!("Alpha must be between 0 and 1");
        }

        BlendCrossover { rate, alpha }
    }
}

impl<C: Chromosome<Gene = FloatGene>> Crossover<C> for BlendCrossover {
    #[inline]
    fn cross_chromosomes(&self, chrom_one: &mut C, chrom_two: &mut C, rate: f32) -> i32 {
        let mut cross_count = 0;

        for i in 0..std::cmp::min(chrom_one.len(), chrom_two.len()) {
            if random_provider::random::<f32>() < rate {
                let gene_one = chrom_one.get_gene(i);
                let gene_two = chrom_two.get_gene(i);

                let allele_one = gene_one.allele();
                let allele_two = gene_two.allele();

                let new_allele_one = allele_one - (self.alpha * (allele_two - allele_one));
                let new_allele_two = allele_two - (self.alpha * (allele_one - allele_two));

                chrom_one.set_gene(i, gene_one.with_allele(&new_allele_one));
                chrom_two.set_gene(i, gene_two.with_allele(&new_allele_two));

                cross_count += 1;
            }
        }

        cross_count
    }
}

impl<C: Chromosome<Gene = FloatGene>> IntoAlter<C> for BlendCrossover {
    fn into_alter(self) -> Alterer<C> {
        Alterer::new(
            "BlendCrossover",
            self.rate,
            AlterAction::Crossover(Box::new(self)),
        )
    }
}
