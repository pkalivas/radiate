use crate::Expr;
use crate::metric_names;

const KP: f32 = 0.05_f32;
const KI: f32 = 0.005_f32;
const KD: f32 = 0.02_f32;

pub fn species_error_signal(count: usize) -> Expr {
    Expr::metric(metric_names::SPECIES_COUNT).error(count as f32)
}

pub fn species_target_control(target: usize, base_val: f32) -> Expr {
    let target_f32 = target as f32;

    let raw_error = Expr::metric(metric_names::SPECIES_COUNT).error(target_f32);

    // Proportional: smoothed count so single-gen bursts don't cause hard jumps
    let proportional = Expr::metric(metric_names::SPECIES_COUNT)
        .rolling(3)
        .mean()
        .error(target_f32)
        * KP;

    // Integral: accumulated recent error over a rolling window
    // Derivative: velocity of the error — anticipates rising/falling count
    let integral = raw_error.clone().rolling(20).sum() * KI;
    let derivative = raw_error.rolling(5).slope() * KD;

    Expr::when(Expr::metric(metric_names::INDEX).lt(2_i32))
        .then(base_val)
        .otherwise(
            Expr::metric(metric_names::SPECIES_THRESHOLD) + proportional + integral + derivative,
        )
        .clamp(0.0_f32, target_f32 * 2.5_f32)
        .alias(metric_names::SPECIES_THRESHOLD)
}

// Rolling slope of best score — useful for limits and convergence detection
pub fn score_trend_signal(window: usize) -> Expr {
    Expr::metric(metric_names::BEST_SCORES)
        .rolling(window)
        .slope()
        .alias(format!("{}.[{}]", metric_names::SCORES_TREND, window))
}

// Coefficient of variation — normalized score spread
pub fn score_cv_signal(window: usize) -> Expr {
    Expr::metric(metric_names::BEST_SCORES)
        .rolling(window)
        .stddev()
        .div(
            Expr::metric(metric_names::BEST_SCORES)
                .rolling(window)
                .mean(),
        )
}

// Throttles add-vertex/add-edge rates as genome grows past target
pub fn genome_size_throttle(base_rate: impl Into<Expr>, target_size: usize) -> Expr {
    let pressure = Expr::metric(metric_names::GENOME_SIZE)
        .rolling(10)
        .mean()
        .div(target_size as f32)
        .clamp(1.0_f32, 5.0_f32);
    base_rate.into().div(pressure)
}

// Higher mutation when diversity is low, lower when healthy
pub fn diversity_signal(window: usize, min: f32, max: f32) -> Expr {
    let diversity = Expr::metric(metric_names::PCT_DIVERSITY)
        .rolling(window)
        .mean();
    (Expr::lit(1.0_f32) - diversity)
        .mul(max - min)
        .add(min)
        .clamp(min, max)
        .alias(format!("{}.[{}]", metric_names::PCT_DIVERSITY, window))
}

// True when best score hasn't meaningfully moved in `window` generations
pub fn stagnation_expr(window: usize, epsilon: f32) -> Expr {
    Expr::metric(metric_names::BEST_SCORES)
        .rolling(window)
        .slope()
        .abs()
        .lt(epsilon)
}

// pub trait MetricExprExt {
//     fn last(self) -> Expr;
//     fn min(self) -> Expr;
//     fn max(self) -> Expr;
//     fn sum(self) -> Expr;
//     fn mean(self) -> Expr;
//     fn variance(self) -> Expr;
//     fn stddev(self) -> Expr;
//     fn skewness(self) -> Expr;
//     fn kurtosis(self) -> Expr;
//     fn count(self) -> Expr;
//     fn generation(self) -> Expr;
//     fn update_count(self) -> Expr;
// }

// impl MetricExprExt for SelectExpr {
//     fn last(self) -> Expr {
//         self.clone().add_attr(metric_fields::LAST_VALUE).into()
//     }

//     fn min(self) -> Expr {
//         self.clone().add_attr(metric_fields::MIN).into()
//     }

//     fn max(self) -> Expr {
//         self.clone().add_attr(metric_fields::MAX).into()
//     }

//     fn sum(self) -> Expr {
//         self.clone().add_attr(metric_fields::SUM).into()
//     }

//     fn mean(self) -> Expr {
//         self.clone().add_attr(metric_fields::MEAN).into()
//     }

//     fn variance(self) -> Expr {
//         self.clone().add_attr(metric_fields::VARIANCE).into()
//     }

//     fn stddev(self) -> Expr {
//         self.clone().add_attr(metric_fields::STDDEV).into()
//     }

//     fn skewness(self) -> Expr {
//         self.clone().add_attr(metric_fields::SKEWNESS).into()
//     }

//     fn kurtosis(self) -> Expr {
//         self.clone().add_attr(metric_fields::KURTOSIS).into()
//     }

//     fn count(self) -> Expr {
//         self.clone().add_attr(metric_fields::COUNT).into()
//     }

//     fn generation(self) -> Expr {
//         self.clone().add_attr(metric_fields::GENERATION).into()
//     }

//     fn update_count(self) -> Expr {
//         self.clone().add_attr(metric_fields::UPDATE_COUNT).into()
//     }
// }
