use super::{AlterAction, AlterResult, Alterer, IntoAlter, Mutate};
use crate::{Chromosome, random_provider};

pub struct ScrambleMutator {
    rate: f32,
}

impl ScrambleMutator {
    pub fn new(rate: f32) -> Self {
        ScrambleMutator { rate }
    }
}

impl<C: Chromosome> Mutate<C> for ScrambleMutator {
    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C, rate: f32) -> AlterResult {
        let mut mutations = 0;

        if random_provider::random::<f32>() < rate {
            let start = random_provider::random_range(0..chromosome.len());
            let end = random_provider::random_range(start..chromosome.len());

            let segment = &mut chromosome.as_mut()[start..end];
            random_provider::shuffle(segment);
            mutations += 1;
        }

        mutations.into()
    }
}

impl<C: Chromosome> IntoAlter<C> for ScrambleMutator {
    fn into_alter(self) -> Alterer<C> {
        Alterer::new(
            "ScrambleMutator",
            self.rate,
            AlterAction::Mutate(Box::new(self)),
        )
    }
}
