use radiate::prelude::*;

const MIN_SCORE: f32 = 0.001;
const TARGET_SPECIES: usize = 4;

fn main() -> RadiateResult<()> {
    random_provider::seed(90);

    let store = node_set! {
        Input => Op::var(0),
        Edge => Op::weight(),
        Vertex => [Op::sub(), Op::mul(), Op::linear()],
        Output => Op::linear()
    };

    let engine = GeneticEngine::builder()
        .codec(GraphCodec::directed(1, 1, store))
        .fitness_fn(Regression::new(DataSet::cubic(), Loss::MSE))
        .minimizing()
        .parallel()
        .metrics([
            expr::score_cv_signal(10).alias("cv"),
            expr::score_trend_signal(10).alias("trend"),
        ])
        .diversity(NeatDistance::new(1.0, 1.0, 3.0))
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

    display(&result);

    Ok(())
}

fn display(result: &Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>) {
    Accuracy::default()
        .named("Regression Graph")
        .on(&DataSet::cubic())
        .loss(Loss::MSE)
        .eval(result.value())
        .inspect(|acc| {
            println!("{result:?}\n{acc:?}\n{}", result.metrics().dashboard());
        });
}
