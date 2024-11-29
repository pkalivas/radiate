use crate::alter::AlterType;
use crate::{random_provider, Alter, Chromosome, Gene};
use std::ops::{Add, Div, Mul, Sub};

pub struct NumericMutator {
    rate: f32,
}

impl NumericMutator {
    pub fn new(rate: f32) -> Self {
        Self { rate }
    }

    pub fn mutate_gene<T>(gene: &T) -> T
    where
        T: Gene + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T>,
    {
        let new_instance = gene.new_instance();
        let operator = random_provider::gen_range(0..4);

        match operator {
            0 => gene.clone() + new_instance,
            1 => gene.clone() - new_instance,
            2 => gene.clone() * new_instance,
            3 => gene.clone() / new_instance,
            _ => panic!("Invalid operator: {}", operator),
        }
    }
}

impl<C: Chromosome> Alter<C> for NumericMutator
where
    C::GeneType: Add<Output = C::GeneType>
        + Sub<Output = C::GeneType>
        + Mul<Output = C::GeneType>
        + Div<Output = C::GeneType>,
{
    fn name(&self) -> &'static str {
        "NumericMutator"
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
        for i in 0..chromosome.len() {
            if random_provider::random::<f32>() < self.rate {
                let curr_gene = chromosome.get_gene(i);
                let new_gene = NumericMutator::mutate_gene(curr_gene);

                chromosome.set_gene(i, new_gene);
                mutations += 1;
            }
        }

        mutations
    }
}
