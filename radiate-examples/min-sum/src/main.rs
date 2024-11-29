use radiate::*;

const MIN_SCORE: i32 = 0;

fn main() {
    let codex = IntCodex::new(1, 10, 0, 100).with_bounds(0, 100);

    let engine = GeneticEngine::from_codex(&codex)
        .population_size(150)
        .minimizing()
        .offspring_selector(EliteSelector::new())
        .survivor_selector(TournamentSelector::new(4))
        .alterer(alters!(
            ArithmeticMutator::new(0.01),
            UniformCrossover::new(0.5),
        ))
        .fitness_fn(|genotype: Vec<Vec<i32>>| {
            Score::from_int(
                genotype
                    .iter()
                    .fold(0, |acc, chromosome| acc + chromosome.iter().sum::<i32>()),
            )
        })
        .build();

    let result = engine.run(|output| {
        println!("[ {:?} ]: {:?}", output.index, output.best.first().unwrap());
        output.score().as_int() == MIN_SCORE
    });

    println!("{:?}", result);
}
