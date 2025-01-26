use radiate::*;
use radiate_gp::*;

const MIN_SCORE: f32 = 0.01;
const MAX_SECONDS: f64 = 1.0;

fn main() {
    random_provider::set_seed(42069);

    let graph_codex = TreeCodex::new(3)
        .constraint(|node| node.size() < 30)
        .gates(vec![Op::add(), Op::sub(), Op::mul()])
        .leafs(vec![Op::var(0)]);

    let regression = Regression::new(get_dataset(), Loss::MSE);

    let engine = GeneticEngine::from_codex(graph_codex)
        .minimizing()
        .num_threads(10)
        .alter(alters!(TreeCrossover::new(0.5)))
        .fitness_fn(move |tree: Tree<f32>| regression.eval(&tree))
        .build();

    let result = engine.run(|ctx| {
        println!("[ {:?} ]: {:?}", ctx.index, ctx.score().as_f32());
        ctx.score().as_f32() < MIN_SCORE || ctx.seconds() > MAX_SECONDS
    });

    display(&result);
}

fn display(result: &EngineContext<TreeChromosome<f32>, Tree<f32>>) {
    let mut regression_accuracy = 0.0;
    let mut total = 0.0;

    let reducer = result.best.clone();
    for sample in get_dataset().iter() {
        let output = reducer.eval(sample.input());

        total += sample.output()[0].abs();
        regression_accuracy += (sample.output()[0] - output).abs();

        println!("{:.2?} :: {:.2?}", sample.output()[0], output);
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
