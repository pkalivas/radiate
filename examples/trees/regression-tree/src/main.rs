use radiate::*;

const MIN_SCORE: f32 = 0.001;

fn main() {
    random_provider::set_seed(518);

    let store = vec![
        (
            NodeType::Vertex,
            vec![Op::add(), Op::sub(), Op::mul(), Op::linear()],
        ),
        (NodeType::Leaf, vec![Op::var(0)]),
    ];

    let tree_codec = TreeCodec::single(3, store).constraint(|root| root.size() < 30);
    let problem = Regression::new(get_dataset(), Loss::MSE);

    println!("{:?}", tree_codec.decode(&tree_codec.encode()));

    let engine = GeneticEngine::builder()
        .codec(tree_codec)
        .fitness_fn(problem)
        .minimizing()
        .mutators(vec![
            Box::new(HoistMutator::new(0.01)),
            // Box::new(OperationMutator::new(0.05, 0.05)),
        ])
        .crossovers(vec![
            Box::new(TreeCrossover::new(0.7)),
            // Box::new(PgmCrossover::new(0.4)),
        ])
        .build();

    engine
        .iter()
        .logging()
        .until_score(MIN_SCORE)
        .last()
        .inspect(display);
}

fn display(result: &Generation<TreeChromosome<Op<f32>>, Tree<Op<f32>>>) {
    let data_set = get_dataset();
    let accuracy = Accuracy::new("reg", &data_set, Loss::MSE);
    let accuracy_result = accuracy.calc(&mut result.value().clone());

    println!("{:?}", result);
    println!("Best Tree: {}", result.value().format());
    println!("{:?}", accuracy_result);
}

fn get_dataset() -> DataSet {
    let mut inputs = Vec::new();
    let mut answers = Vec::new();

    let mut input = -1.0;
    for _ in -10..10 {
        input += 0.1;
        inputs.push(vec![input]);
        answers.push(vec![compute(input)]);
    }

    DataSet::new(inputs, answers)
}

fn compute(x: f32) -> f32 {
    4.0 * x.powf(3.0) - 3.0 * x.powf(2.0) + x
}
