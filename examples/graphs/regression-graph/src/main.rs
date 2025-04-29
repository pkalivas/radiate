use radiate::*;

const MIN_SCORE: f32 = 0.001;
// const MAX_SECONDS: f64 = 5.0;

fn main() {
    random_provider::set_seed(1000);

    let values = vec![
        (NodeType::Input, vec![Op::var(0)]),
        (NodeType::Edge, vec![Op::weight()]),
        (NodeType::Vertex, vec![Op::sub(), Op::mul(), Op::linear()]),
        (NodeType::Output, vec![Op::linear()]),
    ];

    let graph_codex = GraphCodex::directed(1, 1, values);
    let problem = Regression::new(get_dataset(), Loss::MSE, graph_codex);

    let engine = GeneticEngine::from_problem(problem)
        .minimizing()
        .num_threads(10)
        // .diversity(NeatDistance::new(1.0, 1.0, 3.0))
        // .species_threshold(1.8)
        // .max_species_age(25)
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.07, 0.05),
            GraphMutator::new(0.1, 0.1).allow_recurrent(false),
        ))
        .build();

    engine
        .iter()
        .until_score_below(MIN_SCORE)
        .inspect(|ctx| log_ctx!(ctx))
        .last()
        .inspect(display);
}

fn display(result: &Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>) {
    let mut evaluator = GraphEvaluator::new(&result.best);

    let data_set = get_dataset();
    let accuracy = Accuracy::new("reg", &data_set, Loss::MSE);
    let accuracy_result = accuracy.calc(|input| evaluator.eval_mut(input));

    println!("{:?}", result);
    println!("{:?}", accuracy_result);
}

fn get_dataset() -> DataSet {
    let mut inputs = Vec::new();
    let mut answers = Vec::new();

    let mut input = -1.0;
    for _ in -10..10 {
        input += 0.1;
        inputs.push(vec![input]);
        answers.push(vec![compute(input)]);
    }

    DataSet::new(inputs, answers)
}

fn compute(x: f32) -> f32 {
    4.0 * x.powf(3.0) - 3.0 * x.powf(2.0) + x
}
