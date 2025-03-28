use radiate::*;

const MIN_SCORE: i32 = 0;

fn main() {
    let codex = IntCodex::vector(10, 0..100);

    let engine = GeneticEngine::from_codex(codex)
        .population_size(150)
        .minimizing()
        .offspring_selector(EliteSelector::new())
        .alter(alters!(SwapMutator::new(0.05), UniformCrossover::new(0.5)))
        .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
        .build();

    let result = engine.run(|ctx| {
        println!("[ {:?} ]: {:?}", ctx.index, ctx.best);
        ctx.score().as_i32() == MIN_SCORE
    });

    println!("{:?}", result);
}
