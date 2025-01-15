use radiate::*;
use radiate_gp::*;
use random_provider::set_seed;

const MAX_INDEX: i32 = 500;
const MIN_SCORE: f32 = 0.01;

fn main() {
    set_seed(100);
    let graph_codex = GraphCodex::regression(1, 1)
        .with_output(Op::sigmoid())
        .set_nodes(|arc, _| arc.acyclic(1, 1));

    let regression = Regression::new(get_sample_set(), ErrorFunction::MSE);

    let engine = GeneticEngine::from_codex(&graph_codex)
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
        .fitness_fn(move |genotype: Graph<Op<f32>>| {
            let mut reducer = GraphReducer::new(&genotype);
            regression.error(|input| reducer.reduce(input))
        })
        .build();

    let result = engine.run(|output| {
        println!("[ {:?} ]: {:?}", output.index, output.score().as_f32());
        output.index == MAX_INDEX || output.score().as_f32() < MIN_SCORE
    });

    display(&result);
}

fn display(result: &EngineContext<GraphChromosome<Op<f32>>, Graph<Op<f32>>>) {
    let mut reducer = GraphReducer::new(&result.best);
    for sample in get_sample_set().iter() {
        let output = reducer.reduce(&sample.1);
        println!(
            "{:?} -> epected: {:?}, actual: {:.3?}",
            sample.1, sample.2, output
        );
    }

    println!("{:?}", result)
}

fn get_sample_set() -> DataSet {
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

    DataSet::from_vecs(inputs, answers)
}
