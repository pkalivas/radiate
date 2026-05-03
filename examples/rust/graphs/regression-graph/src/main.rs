use radiate::prelude::*;

const MIN_SCORE: f32 = 0.001;

fn main() {
    random_provider::set_seed(90);

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
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.07, 0.05),
            GraphMutator::new(0.1, 0.1).allow_recurrent(false)
        ))
        .build();

    radiate::ui(engine)
        .iter()
        .until_score(MIN_SCORE)
        .last()
        .inspect(display);
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

// random_provider::set_seed(90);

// let store = vec![
//     (NodeType::Input, vec![Op::var(0)]),
//     (NodeType::Edge, vec![Op::weight()]),
//     (NodeType::Vertex, vec![Op::sub(), Op::mul(), Op::linear()]),
//     (NodeType::Output, vec![Op::linear()]),
// ];

// // let target_species = 6.0;
// // let rolling = target_species as usize;

// // let spec_count_signal = expr::select("count.species")
// //     .rolling(rolling)
// //     .mean()
// //     .div(target_species);

// // let spec_dist_signal = expr::select("species.distance")
// //     .mean()
// //     .rolling(rolling)
// //     .mean()
// //     .div(target_species);

// // let spec_thresh_signal = expr::select("species.threshold").rolling(rolling).mean();
// // let spec_evenness_signal = expr::select("species.evenness").rolling(rolling).mean();

// // let distance_signal = spec_count_signal
// //     .mul(0.9)
// //     .add(spec_dist_signal.mul(0.4))
// //     .add(spec_thresh_signal.mul(0.2))
// //     .add(spec_evenness_signal.mul(0.1))
// //     .clamp(0.01, 10.0);

// let engine = GeneticEngine::builder()
//     .codec(GraphCodec::directed(1, 1, store))
//     .raw_batch_fitness_fn(Regression::new(dataset(), Loss::MSE))
//     .minimizing()
//     // .register_metrics(vec![("idk", expr)])
//     // .diversity(NeatDistance::new(1.0, 1.0, 3.0))
//     // .species_threshold(Rate::Expr(distance_signal))
//     .alter(alters!(
//         GraphCrossover::new(0.5, 0.5),
//         OperationMutator::new(0.07, 0.05),
//         GraphMutator::new(0.1, 0.1).allow_recurrent(false)
//     ))
//     .build();
