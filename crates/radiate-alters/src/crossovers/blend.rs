use radiate_core::{AlterResult, ArithmeticGene, Chromosome, Crossover, random_provider};

pub struct BlendCrossover {
    rate: f32,
    alpha: f32,
}

impl BlendCrossover {
    /// Create a new instance of the `BlendCrossover` with the given rate and alpha.
    /// The rate must be between 0.0 and 1.0, and the alpha must be between 0.0 and 1.0.
    pub fn new(rate: f32, alpha: f32) -> Self {
        if !(0.0..=1.0).contains(&rate) {
            panic!("Rate must be between 0 and 1");
        }

        if !(0.0..=1.0).contains(&alpha) {
            panic!("Alpha must be between 0 and 1");
        }

        BlendCrossover { rate, alpha }
    }
}

impl<C, G> Crossover<C> for BlendCrossover
where
    C: Chromosome<Gene = G>,
    G: ArithmeticGene,
    G::Allele: Into<f32> + Clone,
{
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

                let allele_one: f32 = gene_one.allele().clone().into();
                let allele_two: f32 = gene_two.allele().clone().into();

                let new_allele_one = allele_one - (self.alpha * (allele_two - allele_one));
                let new_allele_two = allele_two - (self.alpha * (allele_one - allele_two));

                chrom_one.set(i, gene_one.from_f32(new_allele_one));
                chrom_two.set(i, gene_two.from_f32(new_allele_two));

                cross_count += 1;
            }
        }

        cross_count.into()
    }
}
