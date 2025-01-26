use radiate::*;
use radiate_gp::*;

const MAX_INDEX: i32 = 500;
const MIN_SCORE: f32 = 0.01;

fn main() {
    random_provider::set_seed(501);

    let graph_codex = GraphCodex::acyclic(2, 1).with_output(Op::sigmoid());

    let regression = Regression::new(get_dataset(), Loss::MSE);

    let engine = GeneticEngine::from_codex(graph_codex)
        .minimizing()
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.1, 0.05),
            GraphMutator::new(vec![
                NodeMutate::Edge(0.05, false),
                NodeMutate::Vertex(0.03, false),
            ]),
        ))
        .fitness_fn(move |genotype: Graph<Op<f32>>| {
            let mut reducer = GraphEvaluator::new(&genotype);
            regression.loss(|input| reducer.eval_mut(input))
        })
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
