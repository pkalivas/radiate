use radiate_core::{AlterResult, Chromosome, Crossover, FloatGene, Gene, random_provider};

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
    fn rate(&self) -> f32 {
        self.rate
    }

    #[inline]
    fn cross_chromosomes(&self, chrom_one: &mut C, chrom_two: &mut C, rate: f32) -> AlterResult {
        let mut cross_count = 0;

        for i in 0..std::cmp::min(chrom_one.len(), chrom_two.len()) {
            if random_provider::random::<f32>() < rate {
                let gene_one = chrom_one.get(i);
                let gene_two = chrom_two.get(i);

                let allele_one = gene_one.allele();
                let allele_two = gene_two.allele();

                let alpha = random_provider::range(0.0..self.alpha);
                let allele = allele_one * alpha + allele_two * (1.0 - alpha);

                chrom_one.set(i, gene_one.with_allele(&allele));
                cross_count += 1;
            }
        }

        cross_count.into()
    }
}
