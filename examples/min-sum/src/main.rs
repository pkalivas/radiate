use radiate::*;

const MIN_SCORE: i32 = 0;

fn main() {
    let codec = IntCodec::vector(10, 0..100);

    let mut engine = GeneticEngine::builder()
        .codec(codec)
        .population_size(150)
        .minimizing()
        .offspring_selector(EliteSelector::new())
        .mutator(SwapMutator::new(0.05))
        .crossover(UniformCrossover::new(0.5))
        .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
        .build();

    let result = engine.run(|ctx| {
        println!("[ {:?} ]: {:?}", ctx.index(), ctx.value());
        ctx.score().as_i32() == MIN_SCORE
    });

    println!("{:?}", result);
}
