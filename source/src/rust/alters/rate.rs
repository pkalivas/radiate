use radiate::prelude::*;

fn my_fitness_fn(geno: Vec<f32>) -> f32 {
    geno.iter().sum()
}

fn main() {
    // --8<-- [start:applying]
    // A `rate` parameter accepts anything that implements `Into<Expr>` — a bare
    // numeric literal is the common case (it converts to a constant `Expr`
    // automatically), or build a full `Expr` for something metric-driven.
    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(10, -5.0..5.0))
        .minimizing()
        .fitness_fn(my_fitness_fn)
        .alter(alters![
            GaussianMutator::new(0.1),
            BlendCrossover::new(0.5, 0.5),
        ])
        .build();
    // --8<-- [end:applying]

    // --8<-- [start:constant]
    let mutator = GaussianMutator::new(0.1_f32);
    // --8<-- [end:constant]

    // --8<-- [start:linear_ramp]
    // Ramp from 0.5 down to 0.05 over the first 50 generations, then hold at 0.05
    let (start, end, duration) = (0.5_f32, 0.05_f32, 50.0_f32);
    let progress = Expr::select(metric_names::INDEX)
        .div(duration)
        .clamp(0.0_f32, 1.0_f32);
    let linear_ramp = Expr::lit(start).add(Expr::lit(end).sub(start).mul(progress));
    // --8<-- [end:linear_ramp]

    // --8<-- [start:stepped]
    // Jump to a new rate at fixed generation boundaries
    let index = Expr::select(metric_names::INDEX);
    let stepped = Expr::when(index.clone().lt(25_i32))
        .then(Expr::lit(0.1_f32))
        .otherwise(
            Expr::when(index.lt(75_i32))
                .then(Expr::lit(0.5_f32))
                .otherwise(Expr::lit(0.9_f32)),
        );
    // --8<-- [end:stepped]

    // --8<-- [start:periodic]
    // Oscillate between high and low exploration every 10 generations
    let periodic = Expr::every(10)
        .then(Expr::lit(0.9_f32))
        .otherwise(Expr::lit(0.1_f32));
    // --8<-- [end:periodic]

    // --8<-- [start:exponential_decay]
    // Half-life decay: starts at `start`, halves every `half_life` generations,
    // settling toward `end`
    let (start, end, half_life) = (0.5_f32, 0.05_f32, 25.0_f32);
    let decay = Expr::lit(0.5_f32).pow(Expr::select(metric_names::INDEX).div(half_life));
    let exponential_decay = Expr::lit(end).add(Expr::lit(start).sub(end).mul(decay));
    // --8<-- [end:exponential_decay]

    // --8<-- [start:metric_driven]
    // Boost mutation once the best score has stagnated for 20 generations
    let stagnant = Expr::select("scores.best").stagnation(1e-4_f32).gte(20_u32);
    let stagnation_boost = Expr::when(stagnant)
        .then(Expr::lit(0.30_f32))
        .otherwise(Expr::lit(0.05_f32));

    // Or track a continuous signal: dial mutation down as scores stabilize
    let volatility_driven = Expr::select("score.volatility")
        .rolling(20)
        .mean()
        .clamp(0.01_f32, 0.5_f32);
    // --8<-- [end:metric_driven]
}
