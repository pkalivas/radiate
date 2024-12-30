use radiate::*;
use radiate_extensions::collections::{Graph, GraphChromosome, GraphCodex, GraphReducer, NodeType};
use radiate_extensions::ops::operation;
use radiate_extensions::*;
use random_provider::set_seed;

const MIN_SCORE: f32 = 0.01;
const MAX_SECONDS: f64 = 5.0;

fn main() {
    set_seed(1000);
    let graph_codex = GraphCodex::regression(1, 1)
        .set_outputs(vec![operation::linear()])
        .set_vertices(vec![
            operation::add(),
            operation::sub(),
            operation::mul(),
            operation::sigmoid(),
            operation::tanh(),
        ]);

    let regression = Regression::new(get_sample_set(), ErrorFunction::MSE);

    let engine = GeneticEngine::from_codex(&graph_codex)
        .minimizing()
        .num_threads(1)
        .offspring_selector(RouletteSelector::new())
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            NodeMutator::new(0.07, 0.05),
            GraphMutator::new(vec![
                NodeMutate::Forward(NodeType::Edge, 0.03),
                NodeMutate::Forward(NodeType::Vertex, 0.1),
            ]),
        ))
        .fitness_fn(move |genotype: Graph<f32>| {
            let mut reducer = GraphReducer::new(&genotype);
            Score::from_f32(regression.error(|input| reducer.reduce(input)))
        })
        .build();

    let result = engine.run(|output| {
        println!("[ {:?} ]: {:?}", output.index, output.score().as_float());
        output.score().as_float() < MIN_SCORE || output.seconds() > MAX_SECONDS
    });

    display(&result);
}

fn display(result: &EngineContext<GraphChromosome<f32>, Graph<f32>>) {
    let mut regression_accuracy = 0.0;
    let mut total = 0.0;

    let mut reducer = GraphReducer::new(&result.best);
    for sample in get_sample_set().get_samples().iter() {
        let output = reducer.reduce(&sample.1);

        total += sample.2[0].abs();
        regression_accuracy += (sample.2[0] - output[0]).abs();

        println!("{:.2?} :: {:.2?}", sample.2[0], output[0]);
    }

    regression_accuracy = (total - regression_accuracy) / total;

    println!("Accuracy: {:.2?}", regression_accuracy);
    println!("{:?}", result)
}

fn get_sample_set() -> DataSet<f32> {
    let mut inputs = Vec::new();
    let mut answers = Vec::new();

    let mut input = -1.0;
    for _ in -10..10 {
        input += 0.1;
        inputs.push(vec![input]);
        answers.push(vec![compute(input)]);
    }

    DataSet::from_vecs(inputs, answers)
}

fn compute(x: f32) -> f32 {
    4.0 * x.powf(3.0) - 3.0 * x.powf(2.0) + x
}
