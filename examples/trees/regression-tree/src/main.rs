use radiate::*;

const MIN_SCORE: f32 = 0.01;

fn main() {
    random_provider::set_seed(518);

    let store = vec![
        (NodeType::Vertex, vec![Op::add(), Op::sub(), Op::mul()]),
        (NodeType::Leaf, vec![Op::var(0)]),
    ];

    // let template = Tree::parse("4 * x^3 - 3 * x^2 + x").unwrap();

    // let tree_codex = TreeCodex::single(3, store)
    //     .with_tree(template)
    //     .constraint(|root| root.size() < 30);

    let tree_codex = TreeCodex::single(3, store).constraint(|root| root.size() < 30);
    let problem = Regression::new(get_dataset(), Loss::MSE, tree_codex);

    let engine = GeneticEngine::builder()
        .problem(problem)
        .minimizing()
        .mutator(HoistMutator::new(0.01))
        .crossover(TreeCrossover::new(0.7))
        .build();

    engine
        .iter()
        .until_score_below(MIN_SCORE)
        .inspect(|ctx| log_ctx!(ctx))
        .last()
        .inspect(display);
}

fn display(result: &Generation<TreeChromosome<Op<f32>>, Tree<Op<f32>>>) {
    let data_set = get_dataset();
    let accuracy = Accuracy::new("reg", &data_set, Loss::MSE);
    let accuracy_result = accuracy.calc(|input| vec![result.best.eval(input)]);

    println!("{:?}", result);
    println!("Best Tree: {}", result.best.format());
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
