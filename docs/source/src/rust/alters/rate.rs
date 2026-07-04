use radiate::prelude::*;

fn main() {
    // --8<-- [start:fixed_rate]
    let fixed_rate = Rate::Fixed(0.01);
    // --8<-- [end:fixed_rate]

    // --8<-- [start:linear_rate]
    // Ramp rate down as best score improves
    let _linear_rate = Expr::select("scores.best")
        .rolling(20)
        .mean()
        .clamp(0.01_f32, 0.1_f32);
    // --8<-- [end:linear_rate]

    // --8<-- [start:step_rate]
    // Jump to a lower rate once the score crosses a threshold
    let _step_rate = Expr::when(Expr::select("scores.best").gte(0.9_f32))
        .then(Expr::lit(0.05_f32))
        .otherwise(Expr::lit(0.5_f32));
    // --8<-- [end:step_rate]

    // --8<-- [start:cyclical_rate]
    // Oscillate between high/low exploration every 10 generations
    let _cyclical_rate = Expr::every(10)
        .then(Expr::lit(0.9_f32))
        .otherwise(Expr::lit(0.1_f32));
    // --8<-- [end:cyclical_rate]

    // --8<-- [start:triangular_cyclical_rate]
    // Use rolling score stddev as a diversity-driven rate signal
    let _diversity_rate = Expr::select("scores.best")
        .rolling(50)
        .stddev()
        .clamp(0.01_f32, 0.9_f32);
    // --8<-- [end:triangular_cyclical_rate]

    // --8<-- [start:exponential_rate]
    // Decay rate as score converges toward 1.0
    let _exp_rate = Expr::lit(1.0_f32)
        .sub(Expr::select("scores.best").rolling(10).mean())
        .clamp(0.01_f32, 0.5_f32);
    // --8<-- [end:exponential_rate]
}
