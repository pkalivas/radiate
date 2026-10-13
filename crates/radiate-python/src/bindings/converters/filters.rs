use crate::{
    PyEngineInput, PyEngineInputType,
    bindings::converters::{FromPyEngineInput, Registry},
    registry,
};
use radiate::*;
use std::sync::{Arc, Mutex};

type SharedFilter<C> = Arc<Mutex<dyn EcosystemFilter<C>>>;

pub(crate) fn filters<C: Chromosome + Clone + 'static>() -> Registry<SharedFilter<C>> {
    use crate::constants::components;
    registry!("filter", |f| Arc::new(Mutex::new(f)) as SharedFilter<C>; {
        components::UNIQUE_SCORE_FILTER => UniqueScoreFilter,
    })
}

impl<C: Chromosome + Clone + 'static> FromPyEngineInput for SharedFilter<C> {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Filter)?;
        filters::<C>().convert(input)
    }
}

impl FromPyEngineInput for UniqueScoreFilter {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        let threshold = input.extract::<f64>("threshold")?;
        let max_stagnation = input.extract::<i64>("max_stagnation")?;
        Ok(UniqueScoreFilter::new(
            max_stagnation as usize,
            threshold as f32,
        ))
    }
}
