use radiate::prelude::*;
use std::ops::Range;

fn my_fitness_fn(genotype: &Genotype<FloatChromosome<f32>>) -> Score {
    genotype
        .iter()
        .flat_map(|chromosome| chromosome.iter())
        .map(|gene| *gene.allele())
        .sum::<f32>()
        .into()
}

fn main() {
    // --8<-- [start:problem]
    // Define a problem struct that holds stateful information
    struct MyFloatProblem {
        num_genes: usize,
        value_range: Range<f32>,
    }

    impl Problem<FloatChromosome<f32>, Vec<f32>> for MyFloatProblem {
        fn encode(&self) -> Genotype<FloatChromosome<f32>> {
            Genotype::from(FloatChromosome::from((
                self.num_genes,
                self.value_range.clone(),
            )))
        }

        fn decode(&self, genotype: &Genotype<FloatChromosome<f32>>) -> Vec<f32> {
            genotype
                .iter()
                .flat_map(|chromosome| chromosome.iter())
                .map(|gene| *gene.allele())
                .collect()
        }

        fn eval(&self, genotype: &Genotype<FloatChromosome<f32>>) -> Result<Score, RadiateError> {
            // Evaluate the genotype directly without decoding
            Ok(my_fitness_fn(genotype))
        }
    }

    // `Problem<C, T>` requires `Send + Sync`; this struct satisfies them automatically.
    // You'd only write a manual `unsafe impl` if it held non-thread-safe state.

    // Create an engine with the problem
    let mut engine = GeneticEngine::builder()
        .problem(MyFloatProblem {
            num_genes: 10,
            value_range: 0.0..1.0,
        })
        .build();

    // Run the engine
    let result = engine.run(|epoch| epoch.index() >= 100);
    // --8<-- [end:problem]
}
