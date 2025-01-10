use crate::{random_provider, Chromosome, Gene};
use std::ops::{Add, Div, Mul, Sub};

use super::{Alter, AlterAction, EngineCompoment, Mutate};

/// Arithmetic Mutator. Mutates genes by performing arithmetic operations on them.
/// The ArithmeticMutator takes a rate parameter that determines the likelihood that
/// a gene will be mutated. The ArithmeticMutator can perform addition, subtraction,
/// multiplication, and division on genes.
///
/// This is a simple mutator that can be used with any gene that implements the
/// `Add`, `Sub`, `Mul`, and `Div` traits - `NumericGene` is a good example.
pub struct ArithmeticMutator {
    rate: f32,
}

impl ArithmeticMutator {
    /// Create a new instance of the `ArithmeticMutator` with the given rate.
    /// The rate must be between 0.0 and 1.0.
    pub fn new(rate: f32) -> Self {
        if rate < 0.0 || rate > 1.0 {
            panic!("Rate must be between 0 and 1");
        }

        Self { rate }
    }

    /// Mutate a gene by performing an arithmetic operation on it.
    /// Randomly select a number between 0 and 3, and perform the corresponding
    /// arithmetic operation on the gene.
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

impl<C: Chromosome> Alter<C> for ArithmeticMutator
where
    C::Gene: Add<Output = C::Gene>
        + Sub<Output = C::Gene>
        + Mul<Output = C::Gene>
        + Div<Output = C::Gene>,
{
    fn rate(&self) -> f32 {
        self.rate
    }

    fn to_alter(self) -> AlterAction<C> {
        AlterAction::Mutate(Box::new(self))
    }
}

impl EngineCompoment for ArithmeticMutator {
    fn name(&self) -> &'static str {
        "ArithmeticMutator"
    }
}

impl<C: Chromosome> Mutate<C> for ArithmeticMutator
where
    C::Gene: Add<Output = C::Gene>
        + Sub<Output = C::Gene>
        + Mul<Output = C::Gene>
        + Div<Output = C::Gene>,
{
    fn mutate_chromosome(&self, chromosome: &mut C) -> i32 {
        let mut mutations = 0;
        for i in 0..chromosome.len() {
            if random_provider::random::<f32>() < self.rate {
                let curr_gene = chromosome.get_gene(i);
                let new_gene = ArithmeticMutator::mutate_gene(curr_gene);

                chromosome.set_gene(i, new_gene);
                mutations += 1;
            }
        }

        mutations
    }
}
