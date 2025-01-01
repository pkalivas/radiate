use radiate::*;

use radiate_extensions::*;

const MIN_SCORE: f32 = 0.01;
const MAX_SECONDS: f64 = 5.0;

fn main() {
    // set_seed(200);
    let graph_codex = TreeCodex::new(3)
        .constraint(|node| node.size() < 30)
        .gates(vec![Op::add(), Op::sub(), Op::mul()])
        .leafs(vec![Op::var(0)]);

    let regression = Regression::new(get_sample_set(), ErrorFunction::MSE);

    let engine = GeneticEngine::from_codex(&graph_codex)
        .minimizing()
        .num_threads(10)
        .alter(alters!(TreeCrossover::new(0.5)))
        .fitness_fn(move |genotype: Tree<Op<f32>>| {
            let mut reducer = Tree::new(genotype.root().take().unwrap().to_owned());
            Score::from_f32(regression.error(|input| vec![reducer.reduce(input)]))
        })
        .build();

    let result = engine.run(|output| {
        println!("[ {:?} ]: {:?}", output.index, output.score().as_float());
        output.score().as_float() < MIN_SCORE || output.seconds() > MAX_SECONDS
    });

    display(&result);
}

fn display(result: &EngineContext<TreeChromosome<Op<f32>>, Tree<Op<f32>>>) {
    let mut regression_accuracy = 0.0;
    let mut total = 0.0;

    // let mut reducer = TreeReducer::new(&result.best);
    let mut reducer = result.best.clone();
    for sample in get_sample_set().get_samples().iter() {
        let output = reducer.reduce(&sample.1);

        total += sample.2[0].abs();
        regression_accuracy += (sample.2[0] - output).abs();

        println!("{:.2?} :: {:.2?}", sample.2[0], output);
    }

    regression_accuracy = (total - regression_accuracy) / total;

    println!("Accuracy: {:.2?}", regression_accuracy);
    println!("{:?}", result)
}

fn get_sample_set() -> DataSet {
    let mut inputs = Vec::new();
    let mut answers = Vec::new();

    let mut input = -1.0;
    for _ in -10..10 {
        input += 0.1;
        inputs.push(vec![input]);
        answers.push(vec![compupute(input)]);
    }

    DataSet::from_vecs(inputs, answers)
}

fn compupute(x: f32) -> f32 {
    4.0 * x.powf(3.0) - 3.0 * x.powf(2.0) + x
}

// use radiate::random_provider::set_seed;
// use radiate::*;
// use radiate_extensions::architects::cells::expr;
// use radiate_extensions::*;

// const MIN_SCORE: f32 = 0.01;
// const MAX_SECONDS: f64 = 5.0;

// fn main() {
//     set_seed(200);
//     let graph_codex =
//         TreeCodex::regression(1, 3).set_gates(vec![expr::add(), expr::sub(), expr::mul()]);

//     let regression = Regression::new(get_sample_set(), ErrorFunction::MSE);

//     let engine = GeneticEngine::from_codex(&graph_codex)
//         .minimizing()
//         .num_threads(10)
//         .alter(alters!(
//             TreeCrossover::new(0.5, 10),
//             NodeMutator::new(0.01, 0.05),
//         ))
//         .fitness_fn(move |genotype: Tree<f32>| {
//             let mut reducer = TreeReducer::new(&genotype);
//             Score::from_f32(regression.error(|input| reducer.reduce(input)))
//         })
//         .build();

//     let result = engine.run(|output| {
//         println!("[ {:?} ]: {:?}", output.index, output.score().as_float());
//         output.score().as_float() < MIN_SCORE || output.seconds() > MAX_SECONDS
//     });

//     display(&result);
// }

// fn display(result: &EngineContext<NodeChromosome<f32>, Tree<f32>>) {
//     let mut regression_accuracy = 0.0;
//     let mut total = 0.0;

//     let mut reducer = TreeReducer::new(&result.best);
//     for sample in get_sample_set().get_samples().iter() {
//         let output = reducer.reduce(&sample.1);

//         total += sample.2[0].abs();
//         regression_accuracy += (sample.2[0] - output[0]).abs();

//         println!("{:.2?} :: {:.2?}", sample.2[0], output[0]);
//     }

//     regression_accuracy = (total - regression_accuracy) / total;

//     println!("Accuracy: {:.2?}", regression_accuracy);
//     println!("{:?}", result)
// }

// fn get_sample_set() -> DataSet<f32> {
//     let mut inputs = Vec::new();
//     let mut answers = Vec::new();

//     let mut input = -1.0;
//     for _ in -10..10 {
//         input += 0.1;
//         inputs.push(vec![input]);
//         answers.push(vec![compupute(input)]);
//     }

//     DataSet::from_vecs(inputs, answers)
// }

// fn compupute(x: f32) -> f32 {
//     4.0 * x.powf(3.0) - 3.0 * x.powf(2.0) + x
// }
