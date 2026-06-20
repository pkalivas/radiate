use radiate::*;

const MIN_SCORE: i32 = 0;

fn main() {
    random_provider::seed(42);

    let engine = GeneticEngine::builder()
        .codec(IntCodec::vector(10, 0..100))
        .population_size(150)
        .minimizing()
        .offspring_selector(EliteSelector::new())
        .mutator(SwapMutator::new(0.05))
        .crossover(UniformCrossover::new(0.5))
        .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
        .build();

    let result = engine
        .iter()
        .logging()
        .until(|view| view.score().as_i32() == MIN_SCORE || view.seconds() >= 3.0)
        .run();

    println!("{:?}", result);
}
