use radiate::prelude::*;

const MIN_SCORE: f32 = 0.001;
const SPECIES_THRESHOLD: f32 = 0.3;

fn main() {
    let mut args = std::env::args();

    args.next();
    let distance = args.next().unwrap_or_else(|| "expr".to_string());
    let target_species: usize = args
        .next()
        .unwrap_or_else(|| "4".to_string())
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

fn get_threshold(use_expr_distance: bool, species_count: usize) -> impl Into<Rate> {
    if use_expr_distance {
        let target_species = species_count.max(1).min(100) as f32;
        let rolling = target_species as usize;

        let spec_count_signal = expr::select("count.species")
            .rolling(rolling)
            .mean()
            .div(target_species);

        let spec_dist_signal = expr::select("species.distance")
            .mean()
            .rolling(rolling)
            .mean()
            .div(target_species);

        let spec_thresh_signal = expr::select("species.threshold").rolling(rolling).mean();
        let spec_evenness_signal = expr::select("species.evenness").rolling(rolling).mean();

        Rate::Expr(
            spec_count_signal
                .mul(0.9)
                .add(spec_dist_signal.mul(0.4))
                .add(spec_thresh_signal.mul(0.2))
                .add(spec_evenness_signal.mul(0.1))
                .clamp(0.01, 10.0),
        )
    } else {
        Rate::from(SPECIES_THRESHOLD)
    }
}
