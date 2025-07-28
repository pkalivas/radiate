use crate::{InputConverter, PyEngineInput, PyEngineInputType};
use radiate::Limit;

impl InputConverter<Vec<Limit>> for Vec<PyEngineInput> {
    fn convert(&self) -> Vec<Limit> {
        self.iter().filter_map(|input| input.convert()).collect()
    }
}

impl InputConverter<Option<Limit>> for PyEngineInput {
    fn convert(&self) -> Option<Limit> {
        if self.input_type != PyEngineInputType::Limit {
            return None;
        }

        if let Some(generation) = self.get_usize("generations") {
            return Some(Limit::Generation(generation));
        }

        if let Some(sec) = self.get_f64("seconds") {
            return Some(Limit::Seconds(sec));
        }

        if let Some(score) = self.get_f32("score") {
            return Some(Limit::Score(score));
        }

        None
    }
}
