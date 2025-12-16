use radiate_core::{
    AlterResult, BoundedGene, Chromosome, FloatGene, Gene, Mutate, Rate, random_provider,
};

/// The `JitterMutator` is a simple mutator that adds a small random value to [FloatGene]s.
///
/// The magnitude parameter controls the maximum change that can be applied to a gene.
/// For example, if the magnitude is set to 0.1, then the gene can be changed by a value
/// between -0.1 and 0.1. This allows for fine-tuning of the mutation process,
/// as smaller magnitudes will result in smaller changes to the genes, while larger
/// magnitudes will result in larger changes.
#[derive(Debug, Clone)]
pub struct JitterMutator {
    rate: Rate,
    magnitude: f32,
}

impl JitterMutator {
    pub fn new(rate: impl Into<Rate>, magnitude: f32) -> Self {
        let rate = rate.into();
        if magnitude <= 0.0 {
            panic!("Magnitude must be greater than 0");
        }

        Self { rate, magnitude }
    }
}

impl<C> Mutate<C> for JitterMutator
where
    C: Chromosome<Gene = FloatGene>,
{
    fn rate(&self) -> Rate {
        self.rate.clone()
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C, rate: f32) -> AlterResult {
        let mut count = 0;

        random_provider::with_rng(|rand| {
            for gene in chromosome.genes_mut() {
                if rand.bool(rate) {
                    let change = rand.range(-1.0..1.0) * self.magnitude;
                    let new_allele = gene.allele() + change;
                    let (min, max) = gene.bounds();

                    (*gene.allele_mut()) = new_allele.clamp(*min, *max);
                    count += 1;
                }
            }
        });

        AlterResult::from(count)
    }
}
