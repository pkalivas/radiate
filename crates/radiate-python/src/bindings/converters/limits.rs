use crate::{InputTransform, PyEngineInput, PyEngineInputType, PyMetric};
use pyo3::Python;
use radiate::Limit;
use std::{sync::Arc, time::Duration};

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

        if let Some(generation) = self.get_usize("generations") {
            return Some(Limit::Generation(generation));
        }

        if let Some(sec) = self.get_f64("seconds") {
            return Some(Limit::Seconds(Duration::from_secs_f64(sec)));
        }

        if let Some(score) = self.get_f32("score") {
            return Some(Limit::Score(score.into()));
        }

        if let Some(score) = self.get_vec_f32("score") {
            return Some(Limit::Score(score.into()));
        }

        let window = self.get_usize("window");
        let epsilon = self.get_f32("epsilon");
        if let (Some(window), Some(epsilon)) = (window, epsilon) {
            return Some(Limit::Convergence(window, epsilon));
        }

        let name = self.get_string("name");
        let limit = self.get("limit");
        if let Some(name) = name
            && let Some(limit) = limit
        {
            let limit = limit.clone();
            return Some(Limit::Metric(
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
            ));
        }

        None
    }
}
