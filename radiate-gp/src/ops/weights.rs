use super::Op;
use radiate::{random_provider, Alter, AlterAction, Chromosome, EngineCompoment, Gene, Mutate};
use std::sync::Arc;

const ONE: f32 = 1.0_f32;
const TWO: f32 = 2.0_f32;
const TENTH: f32 = 0.1_f32;

pub struct WeightMutator {
    rate: f32,
}

impl WeightMutator {
    pub fn new(rate: f32) -> Self {
        WeightMutator { rate }
    }
}

impl EngineCompoment for WeightMutator {
    fn name(&self) -> &'static str {
        "WeightMutator"
    }
}

impl<C, G> Alter<C> for WeightMutator
where
    C: Chromosome<Gene = G>,
    G: Gene<Allele = Op<f32>>,
{
    fn rate(&self) -> f32 {
        self.rate
    }

    fn to_alter(self) -> AlterAction<C> {
        AlterAction::Mutate(Box::new(self))
    }
}

impl<C, G> Mutate<C> for WeightMutator
where
    C: Chromosome<Gene = G>,
    G: Gene<Allele = Op<f32>>,
{
    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C) -> i32 {
        let mutation_indexes = (0..chromosome.len())
            .filter(|idx| match chromosome.get_gene(*idx).allele() {
                Op::Weight(_, _, _, _) => true,
                _ => false,
            })
            .filter(|_| random_provider::random::<f32>() < self.rate)
            .collect::<Vec<usize>>();

        let mut count = 0;

        for index in mutation_indexes {
            let genee = chromosome.get_gene(index);
            let new_gene = match genee.allele() {
                Op::Weight(name, arity, value, fn_ptr) => {
                    let modifier = (random_provider::random::<f32>() * TWO - ONE) * TENTH;
                    if random_provider::random::<f32>() < 0.05 {
                        Op::Weight(name, *arity, modifier, Arc::clone(fn_ptr))
                    } else {
                        Op::Weight(name, *arity, value + modifier, Arc::clone(fn_ptr))
                    }
                }
                _ => unreachable!(),
            };

            chromosome.set_gene(index, genee.with_allele(&new_gene));
            count += 1;
        }

        count
    }
}
