use radiate::*;
use radiate_extensions::*;

const MAX_INDEX: i32 = 500;
const MIN_SCORE: f32 = 0.01;

fn main() {
    let graph_codex = GraphCodex::regression(2, 1).set_outputs(vec![op::sigmoid()]);

    let regression = Regression::new(get_sample_set(), ErrorFunction::MSE);

    let engine = GeneticEngine::from_codex(&graph_codex)
        .minimizing()
        .alterer(alters![
            GraphCrossover::new(0.5, 0.5),
            NodeMutator::new(0.01, 0.05),
            GraphMutator::new(vec![
                NodeMutate::Forward(NodeType::Weight, 0.05),
                NodeMutate::Forward(NodeType::Aggregate, 0.03),
                NodeMutate::Forward(NodeType::Gate, 0.03),
            ]),
        ])
        .fitness_fn(move |genotype: Graph<f32>| {
            let mut reducer = GraphReducer::new(&genotype);
            Score::from_f32(regression.error(|input| reducer.reduce(input)))
        })
        .build();

    let result = engine.run(|output| {
        println!("[ {:?} ]: {:?}", output.index, output.score().as_float());
        output.index == MAX_INDEX || output.score().as_float() < MIN_SCORE
    });

    display(&result);
}

fn display(result: &EngineOutput<NodeChromosome<f32>, Graph<f32>>) {
    let mut reducer = GraphReducer::new(&result.best);
    for sample in get_sample_set().get_samples().iter() {
        let output = &reducer.reduce(&sample.1);
        println!(
            "{:?} -> epected: {:?}, actual: {:.3?}",
            sample.1, sample.2, output
        );
    }

    println!("{:?}", result)
}

fn get_sample_set() -> SampleSet<f32> {
    let inputs = vec![
        vec![0.0, 0.0],
        vec![1.0, 1.0],
        vec![1.0, 0.0],
        vec![0.0, 1.0],
    ];

    let answers = vec![vec![0.0], vec![0.0], vec![1.0], vec![1.0]];

    SampleSet::from_vecs(inputs, answers)
}
