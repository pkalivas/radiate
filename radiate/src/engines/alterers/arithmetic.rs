use crate::{Chromosome, Gene, random_provider};
use std::ops::{Add, Div, Mul, Sub};

use super::{AlterAction, Alterer, IntoAlter, Mutate};

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
        let operator = random_provider::random_range(0..4);

        match operator {
            0 => gene.clone() + new_instance,
            1 => gene.clone() - new_instance,
            2 => gene.clone() * new_instance,
            3 => gene.clone() / new_instance,
            _ => panic!("Invalid operator: {}", operator),
        }
    }
}

impl<C: Chromosome> Mutate<C> for ArithmeticMutator
where
    C::Gene: Add<Output = C::Gene>
        + Sub<Output = C::Gene>
        + Mul<Output = C::Gene>
        + Div<Output = C::Gene>,
{
    fn mutate_chromosome(&self, chromosome: &mut C, rate: f32) -> i32 {
        let mut mutations = 0;
        for i in 0..chromosome.len() {
            if random_provider::random::<f32>() < rate {
                let curr_gene = chromosome.get_gene(i);
                let new_gene = ArithmeticMutator::mutate_gene(curr_gene);

                chromosome.set_gene(i, new_gene);
                mutations += 1;
            }
        }

        mutations
    }
}

impl<C: Chromosome> IntoAlter<C> for ArithmeticMutator
where
    C::Gene: Add<Output = C::Gene>
        + Sub<Output = C::Gene>
        + Mul<Output = C::Gene>
        + Div<Output = C::Gene>,
{
    fn into_alter(self) -> Alterer<C> {
        Alterer::new(
            "ArithmeticMutator",
            self.rate,
            AlterAction::Mutate(Box::new(self)),
        )
    }
}
