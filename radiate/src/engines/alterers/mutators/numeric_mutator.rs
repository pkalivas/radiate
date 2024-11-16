use rand::Rng;

use crate::engines::alterers::mutators::mutate::Mutate;
use crate::engines::genome::chromosome::Chromosome;
use crate::engines::genome::genes::gene::NumericGene;

pub struct NumericMutator {
    rate: f32,
}

impl NumericMutator {
    pub fn new(rate: f32) -> Self {
        Self { rate }
    }
}

impl<G, A> Mutate<G, A> for NumericMutator
where
    G: NumericGene<G, A>,
{
    fn mutate_rate(&self) -> f32 {
        self.rate
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut Chromosome<G, A>, _: i32) -> i32 {
        let mut random = rand::thread_rng();
        let mut mutations = 0;

        for gene in chromosome.iter_mut() {
            if random.gen::<f32>() < self.rate {
                let new_instance = gene.new_instance();
                let operator = random.gen_range(0..4);

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
