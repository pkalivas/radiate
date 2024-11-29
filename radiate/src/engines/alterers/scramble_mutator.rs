use crate::alter::AlterType;
use crate::{random_provider, Alter, Chromosome};

pub struct ScrambleMutator {
    rate: f32,
}

impl ScrambleMutator {
    pub fn new(rate: f32) -> Self {
        ScrambleMutator { rate }
    }
}

impl<C: Chromosome> Alter<C> for ScrambleMutator {
    fn name(&self) -> &'static str {
        "ScrambleMutator"
    }

    fn rate(&self) -> f32 {
        self.rate
    }

    fn alter_type(&self) -> AlterType {
        AlterType::Mutator
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C) -> i32 {
        let mut mutations = 0;

        if random_provider::random::<f32>() < self.rate {
            let start = random_provider::gen_range(0..chromosome.len());
            let end = random_provider::gen_range(start..chromosome.len());

            let segment = &mut chromosome.get_genes_mut()[start..end];
            random_provider::shuffle(segment);
            mutations += 1;
        }

        mutations
    }
}