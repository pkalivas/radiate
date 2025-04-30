use radiate_core::{AlterResult, Chromosome, Mutate, random_provider};

pub struct ScrambleMutator {
    rate: f32,
}

impl ScrambleMutator {
    pub fn new(rate: f32) -> Self {
        ScrambleMutator { rate }
    }
}

impl<C: Chromosome> Mutate<C> for ScrambleMutator {
    fn rate(&self) -> f32 {
        self.rate
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C, rate: f32) -> AlterResult {
        let mut mutations = 0;

        if random_provider::random::<f32>() < rate {
            let start = random_provider::range(0..chromosome.len());
            let end = random_provider::range(start..chromosome.len());

            let segment = &mut chromosome.as_mut()[start..end];
            random_provider::shuffle(segment);
            mutations += 1;
        }

        mutations.into()
    }
}
