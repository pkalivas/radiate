use radiate::*;
use radiate_gp::*;

const MAX_INDEX: i32 = 500;
const MIN_SCORE: f32 = 0.01;

fn main() {
    random_provider::set_seed(100);

    let graph_codex = GraphBuilder::default()
        .lstm(1, 1, Op::sigmoid())
        .into_codex();
    let regression = Regression::new(get_dataset(), Loss::MSE);

    let engine = GeneticEngine::from_codex(graph_codex)
        .minimizing()
        .offspring_selector(BoltzmannSelector::new(4_f32))
        .survivor_selector(TournamentSelector::new(4))
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.1, 0.05),
            GraphMutator::new(vec![
                NodeMutate::Edge(0.05, true),
                NodeMutate::Vertex(0.05, true),
            ]),
        ))
        .fitness_fn(move |genotype: Graph<f32>| regression.eval(&genotype))
        .build();

    let result = engine.run(|ctx| {
        println!("[ {:?} ]: {:?}", ctx.index, ctx.score().as_f32());
        ctx.index == MAX_INDEX || ctx.score().as_f32() < MIN_SCORE
    });

    display(&result);
}

fn display(result: &EngineContext<GraphChromosome<f32>, Graph<f32>>) {
    let mut reducer = GraphEvaluator::new(&result.best);
    for sample in get_dataset().iter() {
        let output = reducer.eval_mut(sample.input());
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
        vec![0.0],
        vec![0.0],
        vec![0.0],
        vec![1.0],
        vec![0.0],
        vec![0.0],
        vec![0.0],
    ];

    let answers = vec![
        vec![0.0],
        vec![0.0],
        vec![1.0],
        vec![0.0],
        vec![0.0],
        vec![0.0],
        vec![1.0],
    ];

    DataSet::new(inputs, answers)
}
