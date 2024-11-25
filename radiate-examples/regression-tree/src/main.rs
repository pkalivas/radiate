use radiate::*;
use radiate_extensions::*;

const MIN_SCORE: f32 = 0.01;
const MAX_SECONDS: f64 = 5.0;

fn main() {    
    let factory = NodeFactory::<f32>::regression(1).gates(vec![op::add(), op::sub(), op::mul()]);

    let graph_codex = TreeCodex::new(3, &factory);

    let regression = Regression::new(get_sample_set(), ErrorFunction::MSE);

    let engine = GeneticEngine::from_codex(&graph_codex)
        .minimizing()
        .num_threads(10)
        .alterer(vec![
            TreeCrossover::alterer(0.5, 10),
            NodeMutator::alterer(factory.clone(), 0.001, 0.05),
        ])
        .fitness_fn(move |genotype: Tree<f32>| {
            let mut reducer = TreeReducer::new(&genotype);
            Score::from_f32(regression.error(|input| reducer.reduce(&input)))
        })
        .build();

    let result = engine.run(|output| {
        println!("[ {:?} ]: {:?}", output.index, output.score().as_float());
        output.score().as_float() < MIN_SCORE || output.seconds() > MAX_SECONDS
    });

    display(&result);
}

fn display(result: &EngineContext<Node<f32>, Ops<f32>, Tree<f32>>) {
    let mut regression_accuracy = 0.0;
    let mut total = 0.0;

    let mut reducer = TreeReducer::new(&result.best);
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
