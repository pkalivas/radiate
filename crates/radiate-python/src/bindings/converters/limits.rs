use crate::{InputTransform, PyEngineInput, PyEngineInputType, PyExpr};
use pyo3::Python;
use radiate::Limit;
use std::{collections::VecDeque, time::Duration};

impl InputTransform<Vec<Limit>> for Vec<PyEngineInput> {
    fn transform(&self) -> Vec<Limit> {
        self.iter().filter_map(|input| input.transform()).collect()
    }
}

impl InputTransform<Option<Limit>> for PyEngineInput {
    fn transform(&self) -> Option<Limit> {
        if self.input_type != PyEngineInputType::Limit {
            return None;
        }

        let limit = match self.component() {
            crate::components::SCORE_LIMIT => {
                if let Ok(score) = self.extract::<f64>("score") {
                    Limit::Score(score.into())
                } else if let Ok(score) = self.extract::<Vec<f64>>("score") {
                    Limit::Score(score.into())
                } else {
                    return None;
                }
            }
            crate::components::GENERATIONS_LIMIT => {
                if let Ok(generation) = self.extract::<i64>("generations") {
                    Limit::Generation(generation as usize)
                } else {
                    return None;
                }
            }
            crate::components::SECONDS_LIMIT => {
                if let Ok(sec) = self.extract::<f64>("seconds") {
                    Limit::Seconds(Duration::from_secs_f64(sec))
                } else {
                    return None;
                }
            }
            crate::components::CONVERGENCE_LIMIT => {
                let window = self.extract::<i64>("window").ok();
                let epsilon = self.extract::<f64>("epsilon").ok();
                if let (Some(window), Some(epsilon)) = (window, epsilon) {
                    Limit::Convergence(
                        window as usize,
                        epsilon as f32,
                        VecDeque::with_capacity(window as usize),
                    )
                } else {
                    return None;
                }
            }
            crate::components::EXPR_LIMIT => {
                let expr_limit = self.get("expr");
                {
                    let expr_limit = expr_limit?;
                    return Python::attach(|py| {
                        expr_limit
                            .extract::<PyExpr>(py)
                            .map(|expr| Limit::Expr(expr.into()))
                            .ok()
                    });
                }
            }
            _ => return None,
        };

        Some(limit)
    }
}
