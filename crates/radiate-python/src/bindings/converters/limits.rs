use std::time::Duration;

use crate::{InputTransform, PyEngineInput, PyEngineInputType};
use radiate::Limit;

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

        None
    }
}
