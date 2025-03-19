use super::{AlterAction, AlterResult, Alterer, Crossover, IntoAlter};
use crate::{random_provider, Chromosome, FloatGene, Gene};

/// Intermediate Crossover. This crossover method takes two chromosomes and crosses them
/// by taking a weighted average of the two alleles. The weight is determined by the `alpha`
/// parameter. The new allele is calculated as:
/// ```text
/// a = a1 * alpha + a2 * (1 - alpha)
/// ```
/// where `a` is the new allele, `a1` is the allele from the first chromosome, `a2` is the allele
/// from the second chromosome, and `alpha` is a value between 0 and 1.
///
pub struct IntermediateCrossover {
    rate: f32,
    alpha: f32,
}

impl IntermediateCrossover {
    /// Create a new instance of the `IntermediateCrossover` with the given rate and alpha.
    /// The rate must be between 0.0 and 1.0, and the alpha must be between 0.0 and 1.0.
    pub fn new(rate: f32, alpha: f32) -> Self {
        if !(0.0..=1.0).contains(&rate) {
            panic!("Rate must be between 0 and 1");
        }

        if !(0.0..=1.0).contains(&alpha) {
            panic!("Alpha must be between 0 and 1");
        }

        IntermediateCrossover { rate, alpha }
    }
}

impl<C: Chromosome<Gene = FloatGene>> Crossover<C> for IntermediateCrossover {
    #[inline]
    fn cross_chromosomes(&self, chrom_one: &mut C, chrom_two: &mut C, rate: f32) -> AlterResult {
        let mut cross_count = 0;

        for i in 0..std::cmp::min(chrom_one.len(), chrom_two.len()) {
            if random_provider::random::<f32>() < rate {
                let gene_one = chrom_one.get_gene(i);
                let gene_two = chrom_two.get_gene(i);

                let allele1 = gene_one.allele();
                let allele2 = gene_two.allele();

                let alpha = random_provider::range(0.0..self.alpha);
                let allele = allele1 * alpha + allele2 * (1.0 - alpha);

                chrom_one.set_gene(i, gene_one.with_allele(&allele));
                cross_count += 1;
            }
        }

        cross_count.into()
    }
}

impl<C: Chromosome<Gene = FloatGene>> IntoAlter<C> for IntermediateCrossover {
    fn into_alter(self) -> Alterer<C> {
        Alterer::new(
            "IntermediateCrossover",
            self.rate,
            AlterAction::Crossover(Box::new(self)),
        )
    }
}
