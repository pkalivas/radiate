use std::sync::{Arc, Mutex};

use crate::{InputTransform, PyEngineInput, PyEngineInputType};
use radiate::*;

impl<C> InputTransform<RadiateResult<Vec<Arc<Mutex<dyn EcosystemFilter<C>>>>>>
    for &[PyEngineInput]
where
    C: Chromosome + Clone,
{
    fn transform(&self) -> RadiateResult<Vec<Arc<Mutex<dyn EcosystemFilter<C>>>>> {
        let mut filters: Vec<Arc<Mutex<dyn EcosystemFilter<C>>>> = Vec::new();
        for input in *self {
            if !matches!(input.input_type, PyEngineInputType::Filter) {
                return Err(radiate_err!(Builder: format!(
                    "Expected input type to be Filter, got {:?}",
                    input.input_type
                )));
            }
            match input.component.as_str() {
                crate::constants::UNIQUE_SCORE_FILTER => {
                    let threshold = input.extract::<f64>("threshold")?;
                    let max_stagnation = input.extract::<i64>("max_stagnation")?;
                    filters.push(Arc::new(Mutex::new(UniqueScoreFilter::new(
                        max_stagnation as usize,
                        threshold as f32,
                    ))));
                }
                _ => {
                    return Err(radiate_err!(Builder: format!(
                        "Filter type {} not yet implemented",
                        input.component
                    )));
                }
            }
        }

        Ok(filters)
    }
}
