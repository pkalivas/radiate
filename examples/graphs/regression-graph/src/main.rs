use radiate::{ops::crossover::PgmCrossover, prelude::*};

const MIN_SCORE: f32 = 0.01;

fn main() {
    random_provider::set_seed(1000);

    let tree = TreeNode::new(Op::sigmoid())
        .attach(TreeNode::new(Op::var(0)))
        .attach(TreeNode::new(Op::weight()));

    let store = vec![
        (NodeType::Input, vec![Op::var(0)]),
        (NodeType::Edge, vec![Op::weight()]),
        (
            NodeType::Vertex,
            vec![Op::sub(), Op::mul(), Op::linear(), Op::pgm("pgm", 2, tree)],
        ),
        (NodeType::Output, vec![Op::linear()]),
    ];

    let engine = GeneticEngine::builder()
        .codec(GraphCodec::directed(1, 1, store))
        .fitness_fn(Regression::new(dataset(), Loss::MSE))
        .minimizing()
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.07, 0.05),
            PgmCrossover::new(0.4),
            GraphMutator::new(0.1, 0.1).allow_recurrent(false)
        ))
        .build();

    engine
        .iter()
        .logging()
        .until_score(MIN_SCORE)
        .last()
        .inspect(display);
}

fn display(result: &Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>) {
    let mut evaluator = GraphEvaluator::new(result.value());

    let data_set = dataset().into();
    let accuracy_result = Accuracy::new("reg", &data_set, Loss::MSE).calc(&mut evaluator);

    println!("{result:?}\n{accuracy_result:?}");
}

fn dataset() -> impl Into<DataSet> {
    let mut inputs = Vec::new();
    let mut answers = Vec::new();

    let mut input = -1.0;
    for _ in -10..10 {
        input += 0.1;
        inputs.push(vec![input]);
        answers.push(vec![compute(input)]);
    }

    (inputs, answers)
}

fn compute(x: f32) -> f32 {
    4.0 * x.powf(3.0) - 3.0 * x.powf(2.0) + x
}
