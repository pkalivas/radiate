use radiate::*;

const MAX_INDEX: usize = 500;
const MIN_SCORE: f32 = 0.01;

fn main() -> RadiateResult<()> {
    random_provider::seed(2);

    let values = node_set! {
        Input => Op::var(0),
        Edge => [Op::weight(), Op::identity()],
        Vertex => ops::all_ops(),
        Output => Op::sigmoid()
    };

    let engine = GeneticEngine::builder()
        .codec(GraphCodec::lstm(1, 1, values))
        .fitness_fn(Regression::new(
            DataSet::<f32>::simple_memory(10),
            Loss::MSE,
        ))
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

    display(&result);

    Ok(())
}

fn display(result: &Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>) {
    let dataset = DataSet::<f32>::simple_memory(10);
    let outputs = result.value().eval(&dataset.features());

    for (idx, row) in dataset.iter().enumerate() {
        let output = outputs[idx].clone();
        println!("{:?} -> expected: {:?}, actual: {output:.3?}", row.0, row.1,);
    }

    println!("{result:?}");
}
