use crate::{mutator::ImageMutator, problem::ImageProblem};
use radiate::*;

mod chromosome;
mod mutator;
mod polygon;
mod problem;

// https://medium.com/@sebastian.charmot/genetic-algorithm-for-image-recreation-4ca546454aaa

const NUM_GENES: usize = 175;
const POLYGON_SIZE: usize = 5;

fn main() {
    random_provider::set_seed(50);

    let image_bytes = include_bytes!("../monalisa.png");

    let problem = ImageProblem::new(
        NUM_GENES,
        POLYGON_SIZE,
        image::load_from_memory(image_bytes).unwrap(),
    );

    let engine = GeneticEngine::builder()
        .problem(problem)
        .minimizing()
        .survivor_selector(RouletteSelector::new())
        .offspring_selector(TournamentSelector::new(3))
        .evaluator(WorkerPoolEvaluator::new(10))
        .alter(alters!(
            MeanCrossover::new(0.3),
            ImageMutator::new(0.01, 0.15),
            UniformCrossover::new(0.4)
        ))
        .build();

    let result = engine
        .iter()
        .inspect(|generation| log_ctx!(generation))
        .take(1000)
        .last()
        .unwrap();

    println!("{:?}", result.metrics());

    let save_path = std::env::current_dir().unwrap().join("output.png");
    result.value().save(save_path).unwrap();
}
