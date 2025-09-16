use radiate::*;

const MAX_INDEX: usize = 5000;
const MIN_SCORE: f32 = 0.01;

fn main() {
    random_provider::set_seed(42);

    let values = vec![
        (NodeType::Input, vec![Op::var(0), Op::var(1)]),
        (NodeType::Edge, vec![Op::weight(), Op::identity()]),
        (
            NodeType::Vertex,
            vec![
                Op::add(),
                Op::sub(),
                Op::mul(),
                Op::div(),
                Op::linear(),
                Op::sigmoid(),
            ],
        ),
        (NodeType::Output, vec![Op::sigmoid()]),
    ];

    let graph_codec = GraphCodec::directed(2, 1, values);
    let regression = Regression::new(get_dataset(), Loss::MSE);

    let mut engine = GeneticEngine::builder()
        .codec(graph_codec)
        .fitness_fn(regression)
        .minimizing()
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.05, 0.05),
            GraphMutator::new(0.06, 0.01).allow_recurrent(false),
        ))
        .build();

    let result = engine.run(|ctx| {
        println!("[ {:?} ]: {:?}", ctx.index(), ctx.score().as_f32(),);
        ctx.index() == MAX_INDEX || ctx.score().as_f32() < MIN_SCORE
    });

    display(&result);
}

fn display(result: &Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>) {
    let mut reducer = GraphEvaluator::new(result.value());
    for sample in get_dataset().iter() {
        let output = &reducer.eval_mut(sample.input())[0];
        println!(
            "{:?} -> epected: {:?}, actual: {:.3?}",
            sample.input(),
            sample.output(),
            output
        );
    }

    println!("{result:?}");
}

fn get_dataset() -> DataSet {
    let inputs = vec![
        vec![0.0, 0.0],
        vec![1.0, 1.0],
        vec![1.0, 0.0],
        vec![0.0, 1.0],
    ];

    let answers = vec![vec![0.0], vec![0.0], vec![1.0], vec![1.0]];

    DataSet::new(inputs, answers)
}
