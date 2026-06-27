use radiate::prelude::*;

fn my_fitness_fn(geno: Vec<f32>) -> f32 {
    geno.iter().sum()
}

// Dataset + target function for the speciation showcase (block: example).
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

fn main() {
    // --8<-- [start:building]
    // Select a metric by name (reads last_value by default)
    let score = Expr::select("scores.best");

    // A literal constant
    let threshold = Expr::lit(0.01_f32);
    // --8<-- [end:building]

    // --8<-- [start:aggregations]
    let score = Expr::select("scores.best");

    score.clone().last(); // last recorded value (default)
    score.clone().mean(); // running mean
    score.clone().stddev(); // standard deviation
    score.clone().min(); // running minimum
    score.clone().max(); // running maximum
    score.clone().sum(); // running sum
    score.clone().var(); // variance
    score.clone().skew(); // skewness
    score.clone().count(); // number of values seen
    score.clone().slope(); // linear slope over all accumulated values
    score.clone().unique(); // deduplicated collection

    // Rolling window: aggregate over the last N values only
    score.clone().rolling(50).mean();
    score.clone().rolling(100).slope();
    // --8<-- [end:aggregations]

    // --8<-- [start:comparisons]
    let score = Expr::select("scores.best");
    let index = Expr::select("index");

    score.clone().lt(0.01_f32);
    score.clone().lte(0.01_f32);
    score.clone().gt(0.99_f32);
    score.clone().gte(0.99_f32);
    score.clone().eq(0.5_f32);
    score.clone().ne(0.5_f32);

    // Boolean logic
    score.clone().lt(0.01_f32).and(index.clone().gt(50.0_f32)); // and
    score.clone().lt(0.01_f32).or(index.clone().gt(500.0_f32)); // or
    score.clone().lt(0.01_f32).not(); // not

    // Convenience: between (inclusive)
    score.clone().between(0.0_f32, 1.0_f32);
    // --8<-- [end:comparisons]

    // --8<-- [start:arithmetic]
    let a = Expr::select("scores.best");
    let b = Expr::select("score.volatility");

    a.clone().add(b.clone());
    a.clone().sub(b.clone());
    a.clone().mul(Expr::lit(2.0_f32));
    a.clone().div(b.clone());
    a.clone().pow(Expr::lit(2.0_f32));
    a.clone().neg();
    a.clone().abs();
    a.clone().clamp(Expr::lit(0.0_f32), Expr::lit(1.0_f32));
    // --8<-- [end:arithmetic]

    // --8<-- [start:conditional]
    let expr = Expr::when(Expr::select("scores.best").lt(0.01_f32))
        .then(Expr::select("scores.best").mean())
        .otherwise(Expr::lit(1.0_f32));
    // --8<-- [end:conditional]

    // --8<-- [start:schedule]
    let expr = Expr::every(10)
        .then(Expr::select("scores.best").rolling(10).stddev())
        .otherwise(Expr::select("scores.best"));
    // --8<-- [end:schedule]

    // --8<-- [start:querying]
    // Interpret the time metric as Duration
    Expr::select("time").time().mean();

    // Read count.evaluation as a numeric value
    Expr::select("count.evaluation").count();
    // --8<-- [end:querying]

    // --8<-- [start:limit_expr]
    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(10, -5.0..5.0))
        .minimizing()
        .fitness_fn(my_fitness_fn)
        .build();

    // Stop when the rolling mean of the best score drops below 0.01
    let stop_expr = Expr::select("scores.best").rolling(50).mean().lt(0.01_f32);

    let result = engine
        .iter()
        .limit((
            Limit::Expr(stop_expr),
            Limit::Generation(5000), // hard ceiling
        ))
        .last()
        .unwrap();
    // --8<-- [end:limit_expr]

    // --8<-- [start:derived_metrics]
    let score_trend = Expr::select("scores.best").rolling(20).slope();
    let score_cv = Expr::select("scores.best")
        .rolling(20)
        .stddev()
        .div(Expr::select("scores.best").rolling(20).mean());

    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(10, -5.0..5.0))
        .minimizing()
        .fitness_fn(my_fitness_fn)
        .register_metrics(vec![("score_trend", score_trend), ("score_cv", score_cv)])
        .build();

    let result = engine.iter().limit(5000).last().unwrap();
    let metrics = result.metrics();

    // Access the derived metric values
    if let Some(m) = metrics.get("score_trend") {
        println!("score_trend: {}", m.last_value());
    }
    // --8<-- [end:derived_metrics]

    // --8<-- [start:derived_metrics_limit]
    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(10, -5.0..5.0))
        .minimizing()
        .fitness_fn(my_fitness_fn)
        .register_metrics(vec![(
            "score_trend",
            Expr::select("scores.best").rolling(50).slope(),
        )])
        .build();

    let result = engine
        .iter()
        .limit((
            Limit::Expr(Expr::select("score_trend").abs().lt(0.0001_f32)),
            Limit::Generation(5000),
        ))
        .last()
        .unwrap();
    // --8<-- [end:derived_metrics_limit]

    // --8<-- [start:dynamic_rates]
    let dynamic_rate = Expr::select("score.volatility")
        .rolling(20)
        .mean()
        .clamp(0.01_f32, 0.5_f32);

    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(10, -5.0..5.0))
        .minimizing()
        .fitness_fn(my_fitness_fn)
        .alter(alters![
            GaussianMutator::new(dynamic_rate),
            BlendCrossover::new(0.5, 0.5),
        ])
        .build();
    // --8<-- [end:dynamic_rates]

    // --8<-- [start:example]
    random_provider::seed(90);

    let store = vec![
        (NodeType::Input, vec![Op::var(0)]),
        (NodeType::Edge, vec![Op::weight()]),
        (NodeType::Vertex, vec![Op::sub(), Op::mul(), Op::linear()]),
        (NodeType::Output, vec![Op::linear()]),
    ];

    let target_species = 4.0;
    let rolling = target_species as usize;

    let spec_count_signal = Expr::select("count.species")
        .rolling(rolling)
        .mean()
        .div(target_species);

    let spec_dist_signal = Expr::select("species.distance")
        .mean()
        .rolling(rolling)
        .mean()
        .div(target_species);

    let spec_thresh_signal = Expr::select("species.threshold").rolling(rolling).mean();
    let spec_evenness_signal = Expr::select("species.evenness").rolling(rolling).mean();

    let distance_signal = spec_count_signal
        .mul(0.9)
        .add(spec_dist_signal.mul(0.4))
        .add(spec_thresh_signal.mul(0.2))
        .add(spec_evenness_signal.mul(0.1))
        .clamp(0.01, 10.0);

    let engine = GeneticEngine::builder()
        .codec(GraphCodec::directed(1, 1, store))
        .raw_batch_fitness_fn(Regression::new(dataset(), Loss::MSE))
        .minimizing()
        .diversity(NeatDistance::new(1.0, 1.0, 3.0))
        .species_threshold(Rate::Expr(distance_signal))
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.07, 0.05),
            GraphMutator::new(0.1, 0.1).allow_recurrent(false)
        ))
        .build();
    // --8<-- [end:example]
}
