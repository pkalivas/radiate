use radiate_core::{
    AlterResult, BoundedGene, Chromosome, FloatGene, Gene, Mutate, random_provider,
};

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
