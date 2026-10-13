use radiate::*;

const MAX_INDEX: usize = 5000;
const MIN_SCORE: f32 = 0.01;

fn main() {
    random_provider::seed(42);

    let values = node_set![
        Input => Op::vars(0..2),
        Edge => [Op::weight(), Op::identity()],
        Vertex => [
            Op::add(),
            Op::sub(),
            Op::mul(),
            Op::div(),
            Op::linear(),
            Op::sigmoid(),
        ],
        Output => Op::sigmoid()
    ];

    let graph_codec = GraphCodec::directed(2, 1, values).unwrap();
    let regression = Regression::new(get_dataset(), Loss::MSE);

    let engine = GeneticEngine::builder()
        .codec(graph_codec)
        .fitness_fn(regression)
        .minimizing()
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.05, 0.05),
            GraphMutator::new(0.06, 0.01).allow_recurrent(false),
        ))
        .build();

    engine
        .iter()
        .until(|view| {
            println!("[ {:?} ]: {:?}", view.index(), view.score().as_f32(),);
            view.index() == MAX_INDEX || view.score().as_f32() <= MIN_SCORE
        })
        .run()
        .inspect(display)
        .expect("Engine run failed");
}

fn display(result: &Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>) {
    // let mut reducer = GraphKernel::new(result.value());
    // for sample in get_dataset().iter() {
    //     let output = &reducer.eval_mut(&sample.0)[0];
    //     println!(
    //         "{:?} -> expected: {:?}, actual: {:.3?}",
    //         sample.0, sample.1, output
    //     );
    // }

    println!("{result:?}");
}

fn get_dataset() -> DataSet<f32> {
    let inputs = vec![
        vec![0.0, 0.0],
        vec![1.0, 1.0],
        vec![1.0, 0.0],
        vec![0.0, 1.0],
    ];

    let answers = vec![vec![0.0], vec![0.0], vec![1.0], vec![1.0]];

    DataSet::new(inputs, answers)
}
