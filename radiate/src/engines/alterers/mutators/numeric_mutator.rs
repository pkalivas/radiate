use crate::engines::genome::genes::gene::NumericGene;
use crate::{random_provider, Chromosome, Gene, Mutate};

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
            if random_provider::random::<f32>() < self.rate {
                let new_instance = gene.new_instance();
                let operator = random_provider::gen_range(0..4);

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
