use crate::{mutator::ImageMutator, problem::ImageProblem};
use radiate::{steps::WorkerPoolEvaluator, *};

mod chromosome;
mod mutator;
mod polygon;
mod problem;

const NUM_GENES: usize = 150;
const POLYGON_SIZE: usize = 4;

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
            MeanCrossover::new(0.17),
            ImageMutator::new(0.001, 0.15),
            UniformCrossover::new(0.7)
        ))
        .build();

    let result = engine
        .iter()
        .inspect(|generation| {
            println!(
                "Generation: {}, Best Score: {:?}",
                generation.index(),
                generation.score()
            );
        })
        .take(1000)
        .last()
        .inspect(|generation| {
            println!("Seconds: {:?}", generation.seconds());
        })
        .unwrap();

    let save_path = std::env::current_dir().unwrap().join("output.png");
    result.value().save(save_path).unwrap();
}
