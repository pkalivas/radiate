use radiate_core::{
    AlterResult, BoundedGene, Chromosome, FloatGene, Gene, Mutate, random_provider,
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
    rate: f32,
    magnitude: f32,
}

impl JitterMutator {
    pub fn new(rate: f32, magnitude: f32) -> Self {
        Self { rate, magnitude }
    }
}

impl<C> Mutate<C> for JitterMutator
where
    C: Chromosome<Gene = FloatGene>,
{
    fn rate(&self) -> f32 {
        self.rate
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C, rate: f32) -> AlterResult {
        let mut count = 0;

        for gene in chromosome.genes_mut() {
            if random_provider::random::<f32>() < rate {
                let change = random_provider::range(-1.0..1.0) * self.magnitude;
                let new_allele = gene.allele() + change;
                let bounds = (gene.min(), gene.max());

                (*gene.allele_mut()) = new_allele.clamp(*bounds.0, *bounds.1);
                count += 1;
            }
        }

        count.into()
    }
}
