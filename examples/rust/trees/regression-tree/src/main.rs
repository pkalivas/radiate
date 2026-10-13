use radiate::prelude::*;

const MIN_SCORE: f32 = 0.001;

fn main() -> RadiateResult<()> {
    random_provider::seed(40);

    let data = DataSet::cubic();

    let set = node_set![
        Vertex => [Op::add(), Op::sub(), Op::mul(), Op::linear()],
        Leaf => Op::var(0)
    ];

    let engine = GeneticEngine::builder()
        .codec(TreeCodec::single(3, set).max_size(30))
        .fitness_fn(Regression::new(data, Loss::MSE))
        .minimizing()
        .mutator(HoistMutator::new(0.01))
        .crossover(TreeCrossover::new(0.7))
        .build();

    let result = engine.iter().logging().until_score(MIN_SCORE).last()?;

    display(&result);

    Ok(())
}

fn display(result: &Generation<TreeChromosome<Op<f32>>, Tree<Op<f32>>>) {
    Accuracy::default()
        .named("Regression Tree")
        .on(&DataSet::cubic())
        .loss(Loss::MSE)
        .eval(result.value())
        .inspect(|acc| {
            println!("{}", result.metrics().dashboard());
            println!("Best Tree: {}", result.value().format());
            println!("{:?}", acc);
        });
}
