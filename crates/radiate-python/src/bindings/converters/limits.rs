use crate::{InputTransform, PyEngineInput, PyEngineInputType, PyExpr, PyMetric};
use pyo3::Python;
use radiate::Limit;
use std::{collections::VecDeque, sync::Arc, time::Duration};

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

        let t = match self.component.as_str() {
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
            crate::components::METRIC_LIMIT => {
                let name = self.extract::<String>("name").ok();
                let limit = self.get("limit");
                if let Some(name) = name
                    && let Some(limit) = limit
                {
                    let limit = limit.clone();
                    Limit::Metric(
                        name.clone(),
                        Arc::new(move |metric| {
                            Python::attach(|py| {
                                limit
                                    .inner
                                    .call1(py, (PyMetric::from(metric.clone()),))
                                    .ok()
                                    .and_then(|result| result.extract::<bool>(py).ok())
                                    .unwrap_or(false)
                            })
                        }),
                    )
                } else {
                    return None;
                }
            }
            crate::components::EXPR_LIMIT => {
                let expr_limit = self.get("expr");
                if let Some(expr_limit) = expr_limit {
                    return Python::attach(|py| {
                        expr_limit
                            .extract::<PyExpr>(py)
                            .map(|expr| Limit::Expr(expr.into()))
                            .ok()
                    });
                } else {
                    return None;
                }
            }
            _ => return None,
        };

        // if let Ok(generation) = self.extract::<i64>("generations") {
        //     return Some(Limit::Generation(generation as usize));
        // }

        // if let Ok(sec) = self.extract::<f64>("seconds") {
        //     return Some(Limit::Seconds(Duration::from_secs_f64(sec)));
        // }

        // if let Ok(score) = self.extract::<f64>("score") {
        //     return Some(Limit::Score(score.into()));
        // }

        // if let Ok(score) = self.extract::<Vec<f64>>("score") {
        //     return Some(Limit::Score(score.into()));
        // }

        // let window = self.extract::<i64>("window").ok();
        // let epsilon = self.extract::<f64>("epsilon").ok();
        // if let (Some(window), Some(epsilon)) = (window, epsilon) {
        //     return Some(Limit::Convergence(
        //         window as usize,
        //         epsilon as f32,
        //         VecDeque::with_capacity(window as usize),
        //     ));
        // }

        // let name = self.extract::<String>("name").ok();
        // let limit = self.get("limit");
        // if let Some(name) = name
        //     && let Some(limit) = limit
        // {
        //     let limit = limit.clone();
        //     return Some(Limit::Metric(
        //         name.clone(),
        //         Arc::new(move |metric| {
        //             Python::attach(|py| {
        //                 limit
        //                     .inner
        //                     .call1(py, (PyMetric::from(metric.clone()),))
        //                     .ok()
        //                     .and_then(|result| result.extract::<bool>(py).ok())
        //                     .unwrap_or(false)
        //             })
        //         }),
        //     ));
        // }

        // let expr_limit = self.get("expr");
        // if let Some(expr_limit) = expr_limit {
        //     return Python::attach(|py| {
        //         expr_limit
        //             .extract::<PyExpr>(py)
        //             .map(|expr| Limit::Expr(expr.into()))
        //             .ok()
        //     });
        // }

        Some(t)
    }
}
