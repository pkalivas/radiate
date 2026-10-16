use radiate::prelude::*;

const MIN_SCORE: f32 = 1e-5;
const TARGET_SPECIES: usize = 4;

fn main() -> RadiateResult<()> {
    random_provider::seed(90);

    let data = DataSet::koza1();
    let (_, in_dim, out_dim) = data.shape();

    let store = node_set! {
        Input => Op::vars(0..in_dim),
        Edge => Op::identity(),
        Vertex => [Op::sum(), Op::prod()],
        Output => Op::sum()
    };

    let engine = GeneticEngine::builder()
        .codec(GraphCodec::directed(in_dim, out_dim, store)?.with_max_nodes(10))
        .fitness_fn(Regression::new(data.clone(), Loss::MSE))
        .minimizing()
        .diversity(NeatDistance::default())
        .target_species(TARGET_SPECIES)
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.07, 0.05),
            GraphMutator::new(0.1, 0.1).allow_recurrent(false)
        ))
        .build();

    let result = radiate::ui(engine)
        .iter()
        .logging()
        .until_score(MIN_SCORE)
        .last()?;

    display(&data, &result);

    Ok(())
}

fn display(data: &DataSet<f32>, result: &Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>) {
    Accuracy::default()
        .named("Regression Graph")
        .on(&data)
        .loss(Loss::MSE)
        .eval(result.value())
        .inspect(|acc| {
            println!("{result:?}\n{acc:?}\n{}", result.metrics().dashboard());
        });
}
