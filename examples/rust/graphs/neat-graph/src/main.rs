use radiate::{graphs::NeatDistance, prelude::*};

const MIN_SCORE: f32 = 0.001;
const SPECIES_THRESHOLD: f32 = 0.3;

fn main() {
    let mut args = std::env::args();

    args.next();
    let distance = args.next().unwrap_or_else(|| "expr".to_string());
    let target_species: usize = args
        .next()
        .unwrap_or_else(|| "10".to_string())
        .parse()
        .expect("target_species must be a number");

    let use_expr = distance == "expr";

    random_provider::seed(90);

    let store = vec![
        (NodeType::Input, vec![Op::var(0)]),
        (NodeType::Edge, vec![Op::weight()]),
        (NodeType::Vertex, vec![Op::sub(), Op::mul(), Op::linear()]),
        (NodeType::Output, vec![Op::linear()]),
    ];

    let engine = GeneticEngine::builder()
        .codec(GraphCodec::directed(1, 1, store))
        .raw_batch_fitness_fn(Regression::new(dataset(), Loss::MSE))
        .minimizing()
        // .parallel()
        .diversity(NeatDistance::new(1.0, 1.0, 3.0))
        .species_threshold(get_threshold(use_expr, target_species))
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.07, 0.05),
            GraphMutator::new(0.1, 0.1).allow_recurrent(false)
        ))
        .build();

    // radiate::ui(engine)
    engine.iter().until_score(MIN_SCORE).last().inspect(display);
}

fn display(result: &Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>) {
    Accuracy::default()
        .named("Regression Graph")
        .on(&dataset().into())
        .loss(Loss::MSE)
        .eval(result.value())
        .inspect(|acc| {
            println!("{result:?}\n{acc:?}\n{}", result.metrics().dashboard());
        });
}

fn dataset() -> impl Into<DataSet<f32>> {
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

fn get_threshold(use_expr_distance: bool, target_species: usize) -> impl Into<Rate> {
    if !use_expr_distance {
        return Rate::from(SPECIES_THRESHOLD);
    }

    let target = target_species.clamp(1, 100) as f32;
    let window = (target as usize).max(10);

    let anchor = expr::select("species.distance")
        .max()
        .rolling(window)
        .mean();

    let count_error = expr::select("count.species")
        .rolling(window)
        .mean()
        .sub(target)
        .div(target);

    const GAIN: f32 = 0.999;

    let result = anchor
        .mul(count_error.mul(GAIN).add(1.0))
        .clamp(0.005, 2.0)
        .compile();

    println!("{result:#?}");

    Rate::Expr(result)
}
