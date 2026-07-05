use crate::metric_names;
use radiate_expr::Expr;

const KP: f32 = 0.05_f32;
const KI: f32 = 0.005_f32;
const KD: f32 = 0.02_f32;

pub fn species_error_expr(count: usize) -> Expr {
    Expr::select(metric_names::SPECIES_COUNT).error(count as f32)
}

pub fn target_species_expr(target: usize, base_val: f32) -> Expr {
    let target_f32 = target as f32;

    let raw_error = Expr::select(metric_names::SPECIES_COUNT).error(target_f32);

    // Proportional: smoothed count so single-gen bursts don't cause hard jumps
    let proportional = Expr::select(metric_names::SPECIES_COUNT)
        .rolling(3)
        .mean()
        .error(target_f32)
        * KP;

    // Integral: accumulated recent error over a rolling window
    // Derivative: velocity of the error — anticipates rising/falling count
    let integral = raw_error.clone().rolling(20).sum() * KI;
    let derivative = raw_error.rolling(5).slope() * KD;

    Expr::when(Expr::select(metric_names::INDEX).lt(2_i32))
        .then(base_val)
        .otherwise(
            Expr::select(metric_names::SPECIES_THRESHOLD) + proportional + integral + derivative,
        )
        .clamp(0.0_f32, target_f32 * 2.5_f32)
        .alias(metric_names::SPECIES_THRESHOLD)
}

// Rolling slope of best score — useful for limits and convergence detection
pub fn score_trend_expr(window: usize) -> Expr {
    Expr::select(metric_names::BEST_SCORES)
        .rolling(window)
        .slope()
        .alias(&format!("{}.[{}]", metric_names::SCORES_TREND, window))
}

// Coefficient of variation — normalized score spread
pub fn score_cv_expr(window: usize) -> Expr {
    Expr::select(metric_names::BEST_SCORES)
        .rolling(window)
        .stddev()
        .div(
            Expr::select(metric_names::BEST_SCORES)
                .rolling(window)
                .mean(),
        )
}

// Throttles add-vertex/add-edge rates as genome grows past target
pub fn genome_size_rate(base_rate: impl Into<Expr>, target_size: usize) -> Expr {
    let pressure = Expr::select(metric_names::GENOME_SIZE)
        .rolling(10)
        .mean()
        .div(target_size as f32)
        .clamp(1.0_f32, 5.0_f32);
    base_rate.into().div(pressure)
}

// Higher mutation when diversity is low, lower when healthy
pub fn diversity_driven_rate(window: usize, min: f32, max: f32) -> Expr {
    let diversity = Expr::select(metric_names::DIVERSITY_RATIO)
        .rolling(window)
        .mean();
    (Expr::lit(1.0_f32) - diversity)
        .mul(max - min)
        .add(min)
        .clamp(min, max)
        .alias(format!("{}.[{}]", metric_names::DIVERSITY_RATE, window))
}

// True when best score hasn't meaningfully moved in `window` generations
pub fn stagnation_expr(window: usize, epsilon: f32) -> Expr {
    Expr::select(metric_names::BEST_SCORES)
        .rolling(window)
        .slope()
        .abs()
        .lt(epsilon)
}
