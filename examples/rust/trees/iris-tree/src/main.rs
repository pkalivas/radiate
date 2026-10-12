use radiate::prelude::*;

const MIN_SCORE: f32 = 0.1;
const MAX_SECONDS: f64 = 5.0;
const MIN_TREE_DEPTH: usize = 3;

fn main() {
    random_provider::seed(1000);

    let (mut train, mut test) = DataSet::iris().split(0.75);
    train.standardize();
    test.standardize();

    let (_, in_dim, out_dim) = train.shape();

    let store = vec![
        (NodeType::Root, vec![Op::sigmoid()]),
        (
            NodeType::Vertex,
            vec![Op::sigmoid(), Op::add(), Op::mul(), Op::sub(), Op::dot()],
        ),
        (
            NodeType::Leaf,
            (0..in_dim)
                .map(Op::var)
                .chain(std::iter::once(Op::erc()))
                .collect(),
        ),
    ];

    let engine = GeneticEngine::builder()
        .codec(
            TreeCodec::multi_root(MIN_TREE_DEPTH, out_dim, store)
                .constraint(|node| node.size() < 40),
        )
        .fitness_fn(Regression::new(train.clone(), Loss::MSE))
        .minimizing()
        .alter(alters!(
            TreeCrossover::new(0.7),
            OperationMutator::new(0.01, 0.08),
        ))
        .build();

    engine
        .iter()
        .logging()
        .until(|epoch| epoch.score().as_f32() < MIN_SCORE || epoch.seconds() > MAX_SECONDS)
        .last()
        .inspect(|ctx| display(&train, &test, ctx))
        .unwrap();
}

fn display(
    train: &DataSet<f32>,
    test: &DataSet<f32>,
    result: &Generation<TreeChromosome<Op<f32>>, Vec<Tree<Op<f32>>>>,
) {
    println!("{}", result.metrics().dashboard());
    let mut best = result.value().clone();

    let train_acc = Accuracy::default()
        .named("train")
        .on(train)
        .loss(Loss::MSE)
        .calc(&mut best);
    let test_acc = Accuracy::default()
        .named("test")
        .on(test)
        .loss(Loss::MSE)
        .calc(&mut best);

    println!("{:?}", result);
    println!("{:?}", train_acc);
    println!("{:?}", test_acc);
}
