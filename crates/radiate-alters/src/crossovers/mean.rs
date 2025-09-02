use radiate_core::{AlterResult, ArithmeticGene, Chromosome, Crossover, random_provider};

/// The `MeanCrossover` is a simple crossover method that replaces the genes of the first chromosome
/// with the mean of the two genes. The mean is calculated by adding the two genes together and dividing
/// by two.
///
/// This crossover can only be used with `ArithmeticGene`s and can be largely benifitial. However, keep
/// in mind that because we are taking the mean of two genes, this results in children that
/// converge towards a common distribution. This can be useful in some cases, but it can also
/// result in a loss of diversity in the population in others.
pub struct MeanCrossover {
    rate: f32,
}

impl MeanCrossover {
    /// Create a new instance of the `MeanCrossover` with the given rate.
    /// The rate must be between 0.0 and 1.0.
    pub fn new(rate: f32) -> Self {
        if !(0.0..=1.0).contains(&rate) {
            panic!("The rate must be between 0.0 and 1.0");
        }

        MeanCrossover { rate }
    }
}

impl<C: Chromosome> Crossover<C> for MeanCrossover
where
    C::Gene: ArithmeticGene,
{
    fn rate(&self) -> f32 {
        self.rate
    }

    #[inline]
    fn cross_chromosomes(&self, chrom_one: &mut C, chrom_two: &mut C, rate: f32) -> AlterResult {
        let mut count = 0;

        for (gene_one, gene_two) in chrom_one.iter_mut().zip(chrom_two.iter()) {
            if random_provider::random::<f32>() < rate {
                *gene_one = gene_one.mean(gene_two);
                count += 1;
            }
        }

        count.into()
    }
}
