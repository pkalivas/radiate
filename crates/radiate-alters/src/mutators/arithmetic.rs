use radiate_core::{AlterResult, ArithmeticGene, Chromosome, Mutate, random_provider};

/// Arithmetic Mutator. Mutates genes by performing arithmetic operations on them.
/// The [ArithmeticMutator] takes a rate parameter that determines the likelihood that
/// a gene will be mutated. The [ArithmeticMutator] can perform addition, subtraction,
/// multiplication, and division on genes.
///
/// This is a simple mutator that can be used with any gene that implements the
/// `Add`, `Sub`, `Mul`, and `Div` traits - [ArithmeticGene] is a good example.
#[derive(Debug, Clone)]
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

impl<G: ArithmeticGene, C: Chromosome<Gene = G>> Mutate<C> for ArithmeticMutator {
    fn rate(&self) -> f32 {
        self.rate
    }

    /// Mutate a gene by performing an arithmetic operation on it.
    /// Randomly select a number between 0 and 3, and perform the corresponding
    /// arithmetic operation on the gene.
    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C, rate: f32) -> AlterResult {
        let mut mutations = 0;
        for gene in chromosome.iter_mut() {
            if random_provider::bool(rate) {
                let operator = random_provider::range(0..4);

                let new_gene = match operator {
                    0 => gene.clone() + gene.new_instance(),
                    1 => gene.clone() - gene.new_instance(),
                    2 => gene.clone() * gene.new_instance(),
                    3 => gene.clone() / gene.new_instance(),
                    _ => panic!("Invalid operator - this shouldn't happen: {}", operator),
                };

                *gene = new_gene;
                mutations += 1;
            }
        }

        AlterResult::from(mutations)
    }
}
