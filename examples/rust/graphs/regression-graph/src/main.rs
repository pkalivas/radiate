use radiate::prelude::*;

const MIN_SCORE: f32 = 0.001;

fn main() {
    random_provider::seed(33);

    let store = vec![
        (NodeType::Input, vec![Op::var(0)]),
        (NodeType::Edge, vec![Op::weight()]),
        (NodeType::Vertex, vec![Op::sub(), Op::mul(), Op::linear()]),
        (NodeType::Output, vec![Op::linear()]),
    ];

    let add_node_expr = Expr::when(Expr::select("index").lt(2))
        .then(0.1)
        .otherwise(
            Expr::when(
                Expr::select("genome.size.score.corr")
                    .rolling(20)
                    .mean()
                    .between(0.0, 0.1),
            )
            .then(0.05)
            .otherwise(0.1),
        )
        .alias("test");

    let engine = GeneticEngine::builder()
        .codec(GraphCodec::directed(1, 1, store))
        .raw_batch_fitness_fn(Regression::new(dataset(), Loss::MSE))
        .minimizing()
        .offspring_selector(BoltzmannSelector::new(4.0))
        // .filter(UniqueScoreFilter::new(5, 0.01))
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.07, 0.05),
            GraphMutator::new(0.1, 0.1).allow_recurrent(false)
        ))
        .build();

    // radiate::ui((engine, true))
    engine
        .iter()
        .until_score(MIN_SCORE)
        .last()
        .inspect(display)
        .unwrap();
}

fn display(result: &Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>) {
    // let dot = result.value().to_dot();
    // // Save the DOT representation to a file
    // std::fs::write("graph.dot", dot).expect("Unable to write DOT file");
    Accuracy::default()
        .named("Regression Graph")
        .on(&dataset().into())
        .loss(Loss::MSE)
        .eval(result.value())
        .inspect(|acc| {
            println!("{result:?}\n{acc:?}\n{}", result.metrics().dashboard());
        });
}

fn dataset() -> impl Into<DataSet<f32>> {
    let mut inputs = Vec::new();
    let mut answers = Vec::new();

    let mut input = -1.0;
    for _ in -10..10 {
        input += 0.1;
        inputs.push(vec![input]);
        answers.push(vec![compute(input)]);
    }

    (inputs, answers)
}

fn compute(x: f32) -> f32 {
    4.0 * x.powf(3.0) - 3.0 * x.powf(2.0) + x
}

// let novelty = NoveltySearch::new(|graph: &Graph<Op<f32>>| graph.clone())
//     .k(10)
//     .threshold(0.02)
//     .archive_size(1000)
//     .distance_fn(NeatDistance::new(1.0, 1.0, 3.0));

// let engine = GeneticEngine::builder()
//     .codec(GraphCodec::directed(1, 1, store))
//     // .raw_batch_fitness_fn(Regression::new(dataset(), Loss::MSE))
//     .fitness_fn(novelty)
//     .minimizing()
//     .offspring_selector(BoltzmannSelector::new(4.0))
//     .alter(alters!(
//         GraphCrossover::new(0.5, 0.5),
//         OperationMutator::new(0.07, 0.05),
//         GraphMutator::new(0.1, 0.1).allow_recurrent(false)
//     ))
//     .build();

// radiate::ui(engine)
//     .iter()
//     .take(1000)
//     .last()
//     .inspect(display)
//     .unwrap();
