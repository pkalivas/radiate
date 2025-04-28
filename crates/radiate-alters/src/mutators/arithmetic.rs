use radiate_core::{AlterResult, ArithmeticGene, Chromosome, Gene, Mutate, random_provider};

/// Arithmetic Mutator. Mutates genes by performing arithmetic operations on them.
/// The ArithmeticMutator takes a rate parameter that determines the likelihood that
/// a gene will be mutated. The ArithmeticMutator can perform addition, subtraction,
/// multiplication, and division on genes.
///
/// This is a simple mutator that can be used with any gene that implements the
/// `Add`, `Sub`, `Mul`, and `Div` traits - `ArithmeticGene` is a good example.
pub struct ArithmeticMutator {
    rate: f32,
}

impl ArithmeticMutator {
    /// Create a new instance of the `ArithmeticMutator` with the given rate.
    /// The rate must be between 0.0 and 1.0.
    pub fn new(rate: f32) -> Self {
        if !(0.0..=1.0).contains(&rate) {
            panic!("Rate must be between 0 and 1");
        }

        Self { rate }
    }
}

impl<C: Chromosome> Mutate<C> for ArithmeticMutator
where
    C::Gene: ArithmeticGene,
{
    fn rate(&self) -> f32 {
        self.rate
    }

    /// Mutate a gene by performing an arithmetic operation on it.
    /// Randomly select a number between 0 and 3, and perform the corresponding
    /// arithmetic operation on the gene.
    fn mutate_chromosome(&self, chromosome: &mut C, rate: f32) -> AlterResult {
        let mut mutations = 0;
        for i in 0..chromosome.len() {
            if random_provider::random::<f32>() < rate {
                let curr_gene = chromosome.get(i);
                let new_instance = curr_gene.new_instance();
                let operator = random_provider::range(0..4);

                let new_gene = match operator {
                    0 => curr_gene.clone() + new_instance,
                    1 => curr_gene.clone() - new_instance,
                    2 => curr_gene.clone() * new_instance,
                    3 => curr_gene.clone() / new_instance,
                    _ => panic!("Invalid operator: {}", operator),
                };

                chromosome.set(i, new_gene);
                mutations += 1;
            }
        }

        mutations.into()
    }
}
