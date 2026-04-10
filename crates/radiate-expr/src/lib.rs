mod datatype;
mod expression;

use std::sync::{Arc, Mutex};

pub use datatype::*;
pub use expression::*;

use radiate_core::{MetricSet, Rate};

impl From<Expr> for Rate {
    fn from(expr: Expr) -> Self {
        let mut expr_state = expr.clone();

        let f = move |metrics: &MetricSet| -> f32 {
            expr_state.dispatch(metrics).extract::<f32>().unwrap_or(0.0)
        };

        Rate::Metric(Arc::new(Mutex::new(f)))
    }
}
