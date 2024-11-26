use radiate::*;
use radiate_extensions::*;

const MIN_SCORE: f32 = 0.01;
const MAX_SECONDS: f64 = 5.0;

fn main() {
    
    let graph_codex = GraphCodex::regression(1, 1)
        .set_outputs(vec![op::linear()])
        .set_gates(vec![op::add(), op::sub(), op::mul()]);

    let regression = Regression::new(get_sample_set(), ErrorFunction::MSE);

    let engine = GeneticEngine::from_codex(&graph_codex)
        .minimizing()
        .num_threads(10)
        .alterer(vec![
            GraphCrossover::alterer(0.5, 0.5),
            NodeMutator::alterer(0.01, 0.05),
            GraphMutator::alterer(vec![
                NodeMutate::Forward(NodeType::Weight, 0.05),
                NodeMutate::Forward(NodeType::Aggregate, 0.02),
                NodeMutate::Forward(NodeType::Gate, 0.03),
            ]),
        ])
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

fn display(result: &EngineOutput<NodeChromosome<f32>, Graph<f32>>) {
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

fn get_sample_set() -> SampleSet<f32> {
    let mut inputs = Vec::new();
    let mut answers = Vec::new();

    let mut input = -1.0;
    for _ in -10..10 {
        input += 0.1;
        inputs.push(vec![input]);
        answers.push(vec![compupute(input)]);
    }

    SampleSet::from_vecs(inputs, answers)
}

fn compupute(x: f32) -> f32 {
    4.0 * x.powf(3.0) - 3.0 * x.powf(2.0) + x
}
