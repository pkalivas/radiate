use radiate::*;

const MAX_INDEX: usize = 500;
const MIN_SCORE: f32 = 0.01;

fn main() -> RadiateResult<()> {
    random_provider::seed(2);

    let data = DataSet::<f32>::simple_memory(40);
    let (_, in_dim, out_dim) = data.shape();

    let values = node_set! {
        Input => Op::var(0),
        Edge => [Op::weight(), Op::identity()],
        Vertex => ops::all_ops(),
        Output => Op::sigmoid()
    };

    let engine = GeneticEngine::builder()
        .codec(GraphCodec::lstm(in_dim, out_dim, values))
        .fitness_fn(Regression::new(data.clone(), Loss::MSE))
        .minimizing()
        .offspring_selector(BoltzmannSelector::new(4_f32))
        .survivor_selector(TournamentSelector::new(4))
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.1, 0.05),
            GraphMutator::new(0.05, 0.05)
        ))
        .build();

    let result = engine
        .iter()
        .until(|view| {
            println!("[ {:?} ]: {:?}", view.index(), view.score().as_f32());
            view.index() == MAX_INDEX || view.score().as_f32() < MIN_SCORE
        })
        .last()?;

    display(&data, &result);

    Ok(())
}

fn display(data: &DataSet<f32>, result: &Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>) {
    let outputs = result.value().eval(&data.features());

    for (idx, row) in data.iter().enumerate() {
        let output = outputs[idx].clone();
        println!("{:?} -> expected: {:?}, actual: {output:.3?}", row.0, row.1,);
    }

    println!("{result:?}");
}
