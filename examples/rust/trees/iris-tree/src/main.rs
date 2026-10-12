use radiate::prelude::*;

const MIN_SCORE: f32 = 0.1;
const MAX_SECONDS: f64 = 5.0;
const MIN_TREE_DEPTH: usize = 3;

fn main() -> RadiateResult<()> {
    random_provider::seed(1000);

    let (mut train, mut test) = DataSet::iris().split(0.75);
    train.standardize();
    test.standardize();

    let (_, in_dim, out_dim) = train.shape();

    let set = node_set! {
        Root => Op::sigmoid(),
        Vertex => [Op::sigmoid(), Op::add(), Op::mul(), Op::sub(), Op::dot()],
        Leaf => Op::vars(0..in_dim)
            .into_iter()
            .chain(std::iter::once(Op::erc()))
            .collect::<Vec<_>>()
    };

    let codec =
        TreeCodec::multi_root(MIN_TREE_DEPTH, out_dim, set).constraint(|node| node.size() < 40);

    let engine = GeneticEngine::builder()
        .codec(codec)
        .fitness_fn(Regression::new(train.clone(), Loss::MSE))
        .minimizing()
        .alter(alters!(
            TreeCrossover::new(0.5),
            OperationMutator::new(0.01, 0.08),
            SubtreeMutator::new(0.1),
        ))
        .build();

    let result = engine
        .iter()
        .logging()
        .until(|epoch| epoch.score().as_f32() < MIN_SCORE || epoch.seconds() > MAX_SECONDS)
        .last()?;

    display(&train, &test, &result);

    Ok(())
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
