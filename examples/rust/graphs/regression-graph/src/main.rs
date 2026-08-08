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

    let engine = GeneticEngine::builder()
        .codec(GraphCodec::directed(1, 1, store))
        .raw_batch_fitness_fn(Regression::new(dataset(), Loss::MSE))
        .minimizing()
        .offspring_selector(BoltzmannSelector::new(4.0))
        .parallel()
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.07, 0.05),
            GraphMutator::new(0.1, 0.1).allow_recurrent(false)
        ))
        .build();

    engine
        .iter()
        .logging()
        .until_score(MIN_SCORE)
        .last()
        .inspect(display)
        .unwrap();
}

fn display(result: &Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>) {
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

// engine.on::<EngineStop<Graph<Op<f32>>>>().handle(
//         |event: &EngineStop<Graph<Op<f32>>>, _: &EventContext| {
//             Accuracy::default()
//                 .named("Regression Graph")
//                 .on(&dataset().into())
//                 .loss(Loss::MSE)
//                 .eval(&event.best)
//                 .inspect(|acc| {
//                     println!("{:?}\n{acc:?}\n{}", event.best, event.metrics.dashboard());
//                 });
//         },
//     );
