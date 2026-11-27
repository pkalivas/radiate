use radiate::prelude::*;
use std::time::Instant;

const MIN_SCORE: f32 = 0.1;
const MAX_SECONDS: f64 = 5.0;

fn main() -> RadiateResult<()> {
    random_provider::seed(1000);

    let (mut train, mut test) = DataSet::iris().split(0.75);
    train.standardize();
    test.standardize();

    let (_, in_dim, out_dim) = train.shape();

    let vertices = vec![
        Op::add(),
        Op::sub(),
        Op::mul(),
        Op::sum(),
        Op::gaussian(),
        Op::sigmoid(),
        Op::tanh(),
        Op::relu(),
    ];

    let other = node_set! {
        Input => Op::vars(0..in_dim),
        Edge => [Op::identity(), Op::weight()],
        Vertex => vertices,
        Output => Op::sum()
    };

    let engine = GeneticEngine::builder()
        .codec(GraphCodec::directed(in_dim, out_dim, other)?)
        .fitness_fn(Regression::new(train.clone(), Loss::XEnt))
        .offspring_selector(TournamentSelector::new(4))
        .minimizing()
        // .parallel()
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.02, 0.05),
            GraphMutator::new(0.02, 0.02)
        ))
        .build();

    let lim = Expr::select("count.evaluation")
        .attr("sum")
        .gte(1000)
        .alias("eval limit");

    let start = Instant::now();
    let result = engine
        .iter()
        .logging()
        .limit(lim)
        .until(|epoch| epoch.score().as_f32() < MIN_SCORE || epoch.seconds() > MAX_SECONDS)
        .last()?;
    let elapsed = start.elapsed();
    println!("Elapsed time: {:?}", elapsed);

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
        .loss(Loss::XEnt)
        .eval(result.value());

    let test_acc = Accuracy::default()
        .named("test")
        .on(test)
        .loss(Loss::XEnt)
        .eval(result.value());

    println!("{:?}", result);
    println!("{:?}", train_acc);
    println!("{:?}", test_acc);
}
