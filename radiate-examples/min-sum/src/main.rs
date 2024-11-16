use radiate::engines::alterers::alter::Alterer;
use radiate::engines::alterers::mutators::numeric_mutator::NumericMutator;
use radiate::engines::codexes::int_codex::IntCodex;
use radiate::engines::genetic_engine::GeneticEngine;
use radiate::engines::score::Score;
use radiate::{Elite, Tournament};

const MIN_SCORE: i32 = 0;

fn main() {
    let codex = IntCodex::new(1, 10, 0, 100).with_bounds(0, 100);

    let engine = GeneticEngine::from_codex(&codex)
        .population_size(150)
        .minimizing()
        .offspring_selector(Elite::new())
        .survivor_selector(Tournament::new(4))
        .alterer(vec![
            Alterer::mutation(NumericMutator::new(0.01)),
            Alterer::UniformCrossover(0.5),
        ])
        .fitness_fn(|genotype: Vec<Vec<i32>>| {
            Score::from_int(genotype.iter().fold(0, |acc, chromosome| {
                acc + chromosome.iter().fold(0, |acc, gene| acc + gene)
            }))
        })
        .build();

    let result = engine.run(|output| {
        println!("[ {:?} ]: {:?}", output.index, output.best.first().unwrap());
        output.score().as_int() == MIN_SCORE
    });

    println!("{:?}", result);
}
