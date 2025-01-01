use crate::{random_provider, Chromosome, FloatGene, Gene, NumericGene};

use super::{AlterAction, CrossoverAction, EngineAlterer, EngineCompoment, MutateAction};

pub struct ScrambleMutator {
    rate: f32,
}

impl ScrambleMutator {
    pub fn new(rate: f32) -> Self {
        ScrambleMutator { rate }
    }
}

impl EngineCompoment for ScrambleMutator {
    fn name(&self) -> &'static str {
        "ScrambleMutator"
    }
}

impl<C: Chromosome> EngineAlterer<C> for ScrambleMutator {
    fn rate(&self) -> f32 {
        self.rate
    }

    fn to_alter(self) -> AlterAction<C> {
        AlterAction::Mutate(Box::new(self))
    }
}

impl<C: Chromosome> MutateAction<C> for ScrambleMutator {
    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C) -> i32 {
        let mut mutations = 0;

        if random_provider::random::<f32>() < self.rate {
            let start = random_provider::gen_range(0..chromosome.len());
            let end = random_provider::gen_range(start..chromosome.len());

            let segment = &mut chromosome.as_mut()[start..end];
            random_provider::shuffle(segment);
            mutations += 1;
        }

        mutations
    }
}

// impl<C: Chromosome> Alter<C> for ScrambleMutator {
//     fn name(&self) -> &'static str {
//         "ScrambleMutator"
//     }

//     fn rate(&self) -> f32 {
//         self.rate
//     }

//     fn alter_type(&self) -> AlterType {
//         AlterType::Mutator
//     }

//     #[inline]
//     fn mutate_chromosome(&self, chromosome: &mut C) -> i32 {
//         let mut mutations = 0;

//         if random_provider::random::<f32>() < self.rate {
//             let start = random_provider::gen_range(0..chromosome.len());
//             let end = random_provider::gen_range(start..chromosome.len());

//             let segment = &mut chromosome.as_mut()[start..end];
//             random_provider::shuffle(segment);
//             mutations += 1;
//         }

//         mutations
//     }
// }
