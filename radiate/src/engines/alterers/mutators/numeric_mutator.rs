use crate::engines::alterers::mutators::mutate::Mutate;
use crate::engines::genome::chromosome::Chromosome;
use crate::engines::genome::genes::gene::NumericGene;
use crate::{Gene, RandomProvider};

pub struct NumericMutator {
    rate: f32,
}

impl NumericMutator {
    pub fn new(rate: f32) -> Self {
        Self { rate }
    }
}

impl<C: Chromosome> Mutate<C> for NumericMutator
where
    C::GeneType: NumericGene,
{
    fn mutate_rate(&self) -> f32 {
        self.rate
    }

    fn name(&self) -> &'static str {
        "NumericMutator"
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C, _: i32) -> i32 {
        let mut mutations = 0;

        for gene in chromosome.iter_mut() {
            if RandomProvider::random::<f32>() < self.rate {
                let new_instance = gene.new_instance();
                let operator = RandomProvider::gen_range(0..4);

                mutations += 1;

                *gene = match operator {
                    0 => gene.add(&new_instance),
                    1 => gene.sub(&new_instance),
                    2 => gene.mul(&new_instance),
                    3 => gene.div(&new_instance),
                    _ => panic!("Invalid operator: {}", operator),
                };
            }
        }

        mutations
    }
}
