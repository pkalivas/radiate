use radiate::*;

const MIN_SCORE: i32 = 0;

fn main() {
    random_provider::set_seed(42);

    let mut engine = GeneticEngine::builder()
        .codec(BitCodec::vector(100))
        .population_size(150)
        .minimizing()
        .offspring_selector(EliteSelector::new())
        .mutator(SwapMutator::new(0.05))
        .crossover(UniformCrossover::new(0.5))
        .dist_learner(UmdaBitLearner::new(0.001))
        .fitness_fn(|geno: Vec<bool>| geno.iter().map(|&b| if b { 1 } else { 0 }).sum::<i32>())
        .build();

    let result = engine.run(|ctx| {
        // println!("[ {:?} ]: {:?}", ctx.index(), ctx.value());
        ctx.score().as_i32() == MIN_SCORE || ctx.seconds() > 3.0
    });

    println!("{:?}", result);

    // let mut engine = GeneticEngine::builder()
    //     .codec(IntCodec::vector(10, 0..100))
    //     .population_size(150)
    //     .minimizing()
    //     .offspring_selector(EliteSelector::new())
    //     .mutator(SwapMutator::new(0.05))
    //     .crossover(UniformCrossover::new(0.5))
    //     .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
    //     .build();

    // let result = engine.run(|ctx| {
    //     println!("[ {:?} ]: {:?}", ctx.index(), ctx.value());
    //     ctx.score().as_i32() == MIN_SCORE || ctx.seconds() > 3.0
    // });

    // println!("{:?}", result);
}
