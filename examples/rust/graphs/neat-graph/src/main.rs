use radiate::prelude::*;

const MIN_SCORE: f32 = 0.001;

fn main() {
    let mut args = std::env::args();

    args.next();
    let target_species: usize = args
        .next()
        .unwrap_or_else(|| "4".to_string())
        .parse()
        .expect("target_species must be a number");

    random_provider::seed(90);

    let store = vec![
        (NodeType::Input, vec![Op::var(0)]),
        (NodeType::Edge, vec![Op::weight()]),
        (NodeType::Vertex, vec![Op::sub(), Op::mul(), Op::linear()]),
        (NodeType::Output, vec![Op::linear()]),
    ];

    let engine = GeneticEngine::builder()
        .codec(GraphCodec::directed(1, 1, store))
        .fitness_fn(Regression::new(DataSet::cubic(), Loss::MSE))
        .minimizing()
        .parallel()
        .metrics([
            expr::score_cv_signal(10).alias("cv"),
            expr::score_trend_signal(10).alias("trend"),
        ])
        .diversity(NeatDistance::new(1.0, 1.0, 3.0))
        .target_species(target_species)
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.07, 0.05),
            GraphMutator::new(0.1, 0.1).allow_recurrent(false)
        ))
        .build();

    radiate::ui(engine)
        // engine
        .iter()
        .logging()
        .until_score(MIN_SCORE)
        .last()
        .inspect(display)
        .expect("No result from engine run");
}

fn display(result: &Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>) {
    Accuracy::default()
        .named("Regression Graph")
        .on(&DataSet::cubic())
        .loss(Loss::MSE)
        .eval(result.value())
        .inspect(|acc| {
            println!("{result:?}\n{acc:?}\n{}", result.metrics().dashboard());
        });
}
