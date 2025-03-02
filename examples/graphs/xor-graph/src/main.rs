use radiate::*;
use radiate_gp::*;

const MAX_INDEX: usize = 500;
const MIN_SCORE: f32 = 0.01;

fn main() {
    random_provider::set_seed(501);

    let values = vec![
        (NodeType::Input, vec![Op::var(0), Op::var(1)]),
        (NodeType::Edge, vec![Op::weight(), Op::identity()]),
        (NodeType::Vertex, ops::get_all_operations()),
        (NodeType::Output, vec![Op::sigmoid()]),
    ];

    let graph_codex = GraphCodex::asyclic(2, 1, values);
    let regression = Regression::new(get_dataset(), Loss::MSE);

    let engine = GeneticEngine::from_codex(graph_codex)
        .minimizing()
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.05, 0.05),
            GraphMutator::new(0.06, 0.01).allow_recurrent(false)
        ))
        .fitness_fn(move |genotype: Graph<Op<f32>>| regression.eval(&genotype))
        .build();

    let result = engine.run(|ctx| {
        println!("[ {:?} ]: {:?}", ctx.index, ctx.score().as_f32(),);
        ctx.index == MAX_INDEX || ctx.score().as_f32() < MIN_SCORE
    });

    display(&result);
}

fn display(result: &EngineContext<GraphChromosome<Op<f32>>, Graph<Op<f32>>>) {
    let mut reducer = GraphEvaluator::new(&result.best);
    for sample in get_dataset().iter() {
        let output = &reducer.eval_mut(sample.input())[0];
        println!(
            "{:?} -> epected: {:?}, actual: {:.3?}",
            sample.input(),
            sample.output(),
            output
        );
    }

    println!("{:?}", result)
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
