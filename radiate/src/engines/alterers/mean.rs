use crate::{random_provider, Chromosome, EngineCompoment, NumericGene};

use super::{Alter, AlterAction, Crossover};

/// The `MeanCrossover` is a simple crossover method that replaces the genes of the first chromosome
/// with the mean of the two genes. The mean is calculated by adding the two genes together and dividing
/// by two.
///
/// This crossover can only be used with `NumericGene`s and can be largely benifitial. However, keep
/// in mind that because we are taking the mean of two genes, this results in children that
/// converge towards a common distribution. This can be usefu l in some cases, but it can also
/// result in a loss of diversity in the population in others.
pub struct MeanCrossover {
    rate: f32,
}

impl MeanCrossover {
    /// Create a new instance of the `MeanCrossover` with the given rate.
    /// The rate must be between 0.0 and 1.0.
    pub fn new(rate: f32) -> Self {
        if rate < 0.0 || rate > 1.0 {
            panic!("The rate must be between 0.0 and 1.0");
        }

        MeanCrossover { rate }
    }
}

impl EngineCompoment for MeanCrossover {
    fn name(&self) -> &'static str {
        "Mean Crossover"
    }
}

impl<C: Chromosome> Alter<C> for MeanCrossover
where
    C::Gene: NumericGene,
{
    fn rate(&self) -> f32 {
        self.rate
    }

    fn to_alter(self) -> AlterAction<C> {
        AlterAction::Crossover(Box::new(self))
    }
}

impl<C: Chromosome> Crossover<C> for MeanCrossover
where
    C::Gene: NumericGene,
{
    #[inline]
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
