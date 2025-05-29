mod alter;
mod engine;
mod limit;
mod objective;
mod registry;
mod selector;

pub use alter::*;
#[allow(unused)]
pub(crate) use engine::{build_multi_objective_engine, set_evaluator};
pub use limit::*;
pub(crate) use objective::set_single_objective;
use radiate::Chromosome;
pub use registry::EngineRegistry;
pub(crate) use selector::set_selector;

use crate::PyEngineParam;

pub trait ParamMapper<C: Chromosome> {
    type Output;
    fn map(&self, param: &PyEngineParam) -> Self::Output;
}
