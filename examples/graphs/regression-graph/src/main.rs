use radiate::*;

const MIN_SCORE: f32 = 0.001;

fn main() {
    random_provider::set_seed(1000);

    let store = vec![
        (NodeType::Input, vec![Op::var(0)]),
        (NodeType::Edge, vec![Op::weight()]),
        (NodeType::Vertex, vec![Op::sub(), Op::mul(), Op::linear()]),
        (NodeType::Output, vec![Op::linear()]),
    ];

    let engine = GeneticEngine::builder()
        .codec(GraphCodec::directed(1, 1, store))
        .fitness_fn(Regression::new(dataset(), Loss::MSE))
        .minimizing()
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.07, 0.05),
            GraphMutator::new(0.1, 0.1).allow_recurrent(false)
        ))
        .build();

    engine
        .iter()
        .inspect(|generation| log_gen!(generation))
        .until_score(MIN_SCORE)
        .take(1)
        .last()
        .inspect(display);
}

fn display(result: &Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>) {
    let mut evaluator = GraphEvaluator::new(result.value());

    let data_set = dataset().into();
    let accuracy = Accuracy::new("reg", &data_set, Loss::MSE);
    let accuracy_result = accuracy.calc(|input| evaluator.eval_mut(input));

    println!("{:?}", result);
    println!("{:?}", accuracy_result);
}

fn dataset() -> impl Into<DataSet> {
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

// .diversity(NeatDistance::new(0.1, 0.1, 0.5))
