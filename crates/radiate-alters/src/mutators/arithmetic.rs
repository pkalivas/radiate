use radiate_core::{
    AlterContext, Chromosome, Expr, Mutate, RateSet, chromosomes::NumericGene, random_provider,
};

enum AirthmeticOperator {
    Add,
    Sub,
    Mul,
    Div,
}

const ARITHMETIC_OPTIONS: [AirthmeticOperator; 4] = [
    AirthmeticOperator::Add,
    AirthmeticOperator::Sub,
    AirthmeticOperator::Mul,
    AirthmeticOperator::Div,
];

/// Arithmetic Mutator. Mutates genes by performing arithmetic operations on them.
/// The [ArithmeticMutator] takes a rate parameter that determines the likelihood that
/// a gene will be mutated. The [ArithmeticMutator] can perform addition, subtraction,
/// multiplication, and division on genes.
///
/// This is a simple mutator that can be used with any gene that implements the
/// `Add`, `Sub`, `Mul`, and `Div` traits - [ArithmeticGene] is a good example.
#[derive(Debug, Clone)]
pub struct ArithmeticMutator {
    rate: Expr,
}

impl ArithmeticMutator {
    /// Create a new instance of the `ArithmeticMutator` with the given rate.
    /// The rate must be between 0.0 and 1.0.
    pub fn new(rate: impl Into<Expr>) -> Self {
        Self { rate: rate.into() }
    }
}

impl<C, G, A> Mutate<C> for ArithmeticMutator
where
    C: Chromosome<Gene = G>,
    G: NumericGene<Allele = A>,
{
    fn rates(&self) -> RateSet {
        RateSet::new(self.rate.clone())
    }

    /// Mutate a gene by performing an arithmetic operation on it.
    /// Randomly select a number between 0 and 3, and perform the corresponding
    /// arithmetic operation on the gene.
    #[inline]
    fn mutate_chromosome(&mut self, chromosome: &mut C, ctx: &mut AlterContext) -> usize {
        let mut mutations = 0;

        random_provider::bernoulli_indices(ctx.rate(), chromosome.len(), |i| {
            if let Some(gene) = chromosome.get_mut(i) {
                let operator = random_provider::choose(&ARITHMETIC_OPTIONS);

                let new_gene = match operator {
                    AirthmeticOperator::Add => gene.safe_add(&gene.new_instance()),
                    AirthmeticOperator::Sub => gene.safe_sub(&gene.new_instance()),
                    AirthmeticOperator::Mul => gene.safe_mul(&gene.new_instance()),
                    AirthmeticOperator::Div => gene.safe_div(&gene.new_instance()),
                };

                *gene = new_gene;
                mutations += 1;
            }
        });

        mutations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radiate_core::{
        FloatChromosome, FloatGene, Gene, IntChromosome, IntGene, Valid, alter::AlterUpdates,
    };

    const SAMPLES: usize = 20_000;

    fn mutate<C: Chromosome>(chromosome: &mut C, rate: f32) -> usize
    where
        ArithmeticMutator: Mutate<C>,
    {
        let mut updates = AlterUpdates::default();
        let mut ctx = AlterContext::new(&mut updates, 0, rate, &[]);
        ArithmeticMutator::new(rate).mutate_chromosome(chromosome, &mut ctx)
    }

    /// Apply the mutator to a single gene with `rate = 1`, so exactly one operator runs.
    fn mutate_float(gene: &FloatGene<f32>) -> f32 {
        let mut chromosome = FloatChromosome::from(gene.clone());
        assert_eq!(mutate(&mut chromosome, 1.0), 1);
        *chromosome.iter().next().unwrap().allele()
    }

    fn mutate_int(gene: &IntGene<i8>) -> i8 {
        let mut chromosome = IntChromosome::from(gene.clone());
        assert_eq!(mutate(&mut chromosome, 1.0), 1);
        *chromosome.iter().next().unwrap().allele()
    }

    #[test]
    fn zero_rate_leaves_chromosome_unchanged() {
        random_provider::seed(1);

        let original = FloatChromosome::<f32>::from((100, -10.0..10.0));
        let mut chromosome = original.clone();

        assert_eq!(mutate(&mut chromosome, 0.0), 0);
        assert!(chromosome == original);
    }

    #[test]
    fn full_rate_mutates_every_float_gene() {
        random_provider::seed(2);

        let original = FloatChromosome::<f32>::from((100, -10.0..10.0, -1000.0..1000.0));
        let mut chromosome = original.clone();

        // Every gene is selected. A handful may land back on their old value
        // (e.g. clamping), so only require that nearly all of them changed.
        assert_eq!(mutate(&mut chromosome, 1.0), 100);

        let changed = chromosome
            .iter()
            .zip(original.iter())
            .filter(|(new, old)| new.allele() != old.allele())
            .count();
        assert!(changed >= 95, "only {changed} of 100 genes changed");
    }

    #[test]
    fn full_rate_mutates_every_int_gene() {
        random_provider::seed(3);

        let original = IntChromosome::<i32>::from((100, -10..10, -1000..1000));
        let mut chromosome = original.clone();

        // Integers can legitimately land on the same value (x * 1, x / 1, x + 0),
        // so the bar is lower than for floats.
        assert_eq!(mutate(&mut chromosome, 1.0), 100);

        let changed = chromosome
            .iter()
            .zip(original.iter())
            .filter(|(new, old)| new.allele() != old.allele())
            .count();
        assert!(changed >= 75, "only {changed} of 100 genes changed");
    }

    #[test]
    fn rate_is_applied_per_gene() {
        random_provider::seed(4);

        let rate = 0.3;
        let trials = 1_000;
        let total: usize = (0..trials)
            .map(|_| mutate(&mut FloatChromosome::<f32>::from((100, -10.0..10.0)), rate))
            .sum();

        let mean = total as f32 / trials as f32;
        assert!(
            (mean - 30.0).abs() < 1.0,
            "mean mutations per chromosome = {mean}"
        );
    }

    #[test]
    fn all_four_operators_are_applied_evenly() {
        random_provider::seed(5);

        // With the allele at 10 and operands drawn from [2, 3), each operator
        // lands in its own disjoint interval, so the result identifies the operator.
        let gene = FloatGene::new(10.0_f32, 2.0..3.0, -1000.0..1000.0);

        let mut counts = [0usize; 4];
        for _ in 0..SAMPLES {
            let y = mutate_float(&gene);
            let op = match y {
                y if (12.0..=13.0).contains(&y) => 0, // add
                y if (7.0..=8.0).contains(&y) => 1,   // sub
                y if (20.0..=30.0).contains(&y) => 2, // mul
                y if (3.3..=5.0).contains(&y) => 3,   // div
                _ => panic!("result {y} does not match any operator"),
            };
            counts[op] += 1;
        }

        for (op, count) in ["add", "sub", "mul", "div"].iter().zip(counts) {
            let frac = count as f32 / SAMPLES as f32;
            assert!(
                (frac - 0.25).abs() < 0.02,
                "{op} applied {frac} of the time"
            );
        }
    }

    #[test]
    fn results_are_clamped_to_bounds() {
        random_provider::seed(6);

        // Operands from [-100, 100) overshoot the [0, 1] bounds on almost every draw.
        let float_gene = FloatGene::new(0.5_f32, -100.0..100.0, 0.0..1.0);
        for _ in 0..SAMPLES {
            let y = mutate_float(&float_gene);
            assert!((0.0..=1.0).contains(&y), "float mutated out of bounds: {y}");
            assert!(float_gene.with_allele(&y).is_valid());
        }

        let int_gene = IntGene::new(5_i8, -100..100, 0..10);
        for _ in 0..SAMPLES {
            let y = mutate_int(&int_gene);
            assert!((0..=10).contains(&y), "int mutated out of bounds: {y}");
            assert!(int_gene.with_allele(&y).is_valid());
        }
    }

    #[test]
    fn integer_overflow_saturates() {
        random_provider::seed(7);

        // At either end of i8, add/sub/mul with operands in [100, 127) overflow.
        // They should saturate at the bound instead of panicking or wrapping.
        for allele in [i8::MAX, i8::MIN] {
            let gene = IntGene::new(allele, 100..127, i8::MIN..i8::MAX);
            for _ in 0..SAMPLES {
                let y = mutate_int(&gene);
                match allele {
                    i8::MAX => assert!(y >= 0, "{allele} wrapped to {y}"),
                    _ => assert!(y <= 0, "{allele} wrapped to {y}"),
                }
            }
        }
    }

    #[test]
    fn division_by_zero_is_a_no_op() {
        random_provider::seed(8);

        // Every operand is 0: add, sub, and div leave the gene at 50, mul zeroes it.
        let gene = IntGene::new(50_i8, 0..1, -100..100);

        let mut unchanged = 0;
        for _ in 0..SAMPLES {
            match mutate_int(&gene) {
                50 => unchanged += 1,
                0 => {}
                y => panic!("unexpected result {y}"),
            }
        }

        let frac = unchanged as f32 / SAMPLES as f32;
        assert!((frac - 0.75).abs() < 0.02, "unchanged fraction = {frac}");
    }
}
