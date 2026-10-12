use radiate::prelude::*;

const MIN_SCORE: f32 = 0.1;
const MAX_SECONDS: f64 = 5.0;

fn main() -> RadiateResult<()> {
    random_provider::seed(1000);

    let (mut train, mut test) = DataSet::iris().split(0.75);
    train.standardize();
    test.standardize();

    let (_, in_dim, out_dim) = train.shape();

    let ops = ops::all_ops();
    let edges = vec![Op::identity(), Op::weight()];
    let outputs = vec![Op::sigmoid()];

    let store = vec![
        (NodeType::Input, (0..in_dim).map(Op::var).collect()),
        (NodeType::Edge, edges.clone()),
        (NodeType::Vertex, ops.clone()),
        (NodeType::Output, outputs.clone()),
    ];

    let codec = GraphCodec::directed(in_dim, out_dim, store);
    let regression = Regression::new(train.clone(), Loss::MSE);

    let engine = GeneticEngine::builder()
        .codec(codec)
        .minimizing()
        .fitness_fn(regression)
        .parallel()
        .offspring_fraction(0.9)
        .replace_strategy(GraphReplacement)
        .offspring_selector(BoltzmannSelector::new(4.0))
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.02, 0.05),
            GraphMutator::new(0.008, 0.002)
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
    result: &Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>,
) {
    println!("{}", result.metrics().dashboard());

    let train_acc = Accuracy::default()
        .named("train")
        .on(train)
        .loss(Loss::MSE)
        .eval(result.value());

    let test_acc = Accuracy::default()
        .named("test")
        .on(test)
        .loss(Loss::MSE)
        .eval(result.value());

    println!("{:?}", result);
    println!("{:?}", train_acc);
    println!("{:?}", test_acc);
}
