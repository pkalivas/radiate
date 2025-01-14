use radiate::*;

const MIN_SCORE: i32 = 0;

fn main() {
    let codex = IntCodex::new(1, 10, 0, 100).with_bounds(0, 100);

    let engine = GeneticEngine::from_codex(&codex)
        .population_size(150)
        .minimizing()
        .offspring_selector(EliteSelector::new())
        .alter(alters!(SwapMutator::new(0.05), UniformCrossover::new(0.5)))
        .fitness_fn(|geno: Vec<Vec<i32>>| geno.iter().flatten().sum::<i32>())
        .build();

    let result = engine.run(|output| {
        println!("[ {:?} ]: {:?}", output.index, output.best.first().unwrap());
        output.score().as_i32() == MIN_SCORE
    });

    println!("{:?}", result);
}
