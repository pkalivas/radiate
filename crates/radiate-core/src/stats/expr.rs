use crate::metric_names;
use radiate_expr::{Expr, NamedExpr};

const KP: f32 = 0.05_f32;
const KI: f32 = 0.005_f32;
const KD: f32 = 0.02_f32;

pub fn species_error_expr(count: usize) -> NamedExpr {
    Expr::select(metric_names::SPECIES_COUNT)
        .error(count as f32)
        .alias(metric_names::SPECIES_ERROR)
}

pub fn target_species_expr(target: usize, base_val: f32) -> NamedExpr {
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
