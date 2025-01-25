use radiate::*;
use radiate_gp::*;

const MIN_SCORE: f32 = 0.001;
const MAX_SECONDS: f64 = 5.0;

fn main() {
    random_provider::set_seed(1000);

    let graph_codex = GraphCodex::regression(1, 1)
        .with_vertices(vec![Op::add(), Op::sub(), Op::mul()])
        .with_output(Op::linear());

    let regression = Regression::new(get_dataset(), Loss::MSE);

    let engine = GeneticEngine::from_codex(&graph_codex)
        .minimizing()
        .num_threads(10)
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.07, 0.05),
            GraphMutator::new(vec![
                NodeMutate::Edge(0.03, false),
                NodeMutate::Vertex(0.1, false),
            ]),
        ))
        .fitness_fn(move |genotype: Graph<Op<f32>>| {
            let mut reducer = GraphReducer::new(&genotype);
            regression.loss(|input| reducer.reduce(input))
        })
        .build();

    let result = engine.run(|ctx| {
        println!("[ {:?} ]: {:?}", ctx.index, ctx.score().as_f32());
        ctx.score().as_f32() < MIN_SCORE || ctx.seconds() > MAX_SECONDS
    });

    display(&result);
}

fn display(result: &EngineContext<GraphChromosome<Op<f32>>, Graph<Op<f32>>>) {
    let mut regression_accuracy = 0.0;
    let mut total = 0.0;

    let mut reducer = GraphReducer::new(&result.best);
    for sample in get_dataset().iter() {
        let output = reducer.reduce(sample.input());

        total += sample.output()[0].abs();
        regression_accuracy += (sample.output()[0] - output[0]).abs();

        println!("{:.2?} :: {:.2?}", sample.output()[0], output[0]);
    }

    regression_accuracy = (total - regression_accuracy) / total;

    println!("Accuracy: {:.2?}", regression_accuracy);
    println!("{:?}", result)
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
