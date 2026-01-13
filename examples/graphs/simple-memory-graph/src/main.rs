use radiate::*;

const MAX_INDEX: usize = 500;
const MIN_SCORE: f32 = 0.01;

fn main() {
    random_provider::set_seed(2);

    let values = vec![
        (NodeType::Input, vec![Op::var(0)]),
        (NodeType::Edge, vec![Op::weight(), Op::identity()]),
        (NodeType::Vertex, ops::all_ops()),
        (NodeType::Output, vec![Op::sigmoid()]),
    ];

    let mut engine = GeneticEngine::builder()
        .codec(GraphCodec::directed(1, 1, values))
        .fitness_fn(Regression::new(dataset(), Loss::MSE))
        .minimizing()
        .offspring_selector(BoltzmannSelector::new(4_f32))
        .survivor_selector(TournamentSelector::new(4))
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.1, 0.05),
            GraphMutator::new(0.05, 0.05)
        ))
        .build();

    let result = engine.run(|ctx| {
        println!("[ {:?} ]: {:?}", ctx.index(), ctx.score().as_f32());
        ctx.index() == MAX_INDEX || ctx.score().as_f32() < MIN_SCORE
    });

    display(&result);
}

fn display(result: &Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>) {
    // save result to json
    let json = serde_json::to_string_pretty(result.value()).unwrap();
    std::fs::write("result.json", json).unwrap();

    let dataset = dataset();
    let outputs = result.value().eval(&dataset.features());

    for (idx, row) in dataset.iter().enumerate() {
        let output = outputs[idx].clone();
        println!(
            "{:?} -> expected: {:?}, actual: {output:.3?}",
            row.input(),
            row.output(),
        );
    }

    println!("{result:?}");
}

fn dataset() -> DataSet<f32> {
    DataSet::default()
        .row((vec![0.0], vec![0.0]))
        .row((vec![0.0], vec![0.0]))
        .row((vec![0.0], vec![1.0]))
        .row((vec![1.0], vec![0.0]))
        .row((vec![0.0], vec![0.0]))
        .row((vec![0.0], vec![0.0]))
        .row((vec![0.0], vec![1.0]))
}
