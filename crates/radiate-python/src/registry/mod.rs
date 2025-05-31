mod alters;
mod diversity;
mod engine;
mod limit;
mod objective;
mod registry;
mod selector;

pub use alters::*;
pub use diversity::*;
pub(crate) use engine::set_evaluator;
pub use limit::*;
pub(crate) use objective::{set_multi_objective, set_single_objective};
use radiate::{Chromosome, Epoch, GeneticEngineBuilder};
pub use registry::{ComponentRegistry, EngineRegistry};

use crate::{PyEngineBuilder, PyEngineParam};

pub trait ParamMapper {
    type Output;
    fn map(&self, param: &PyEngineParam) -> Self::Output;
}

pub trait EngineParamTransform<C: Chromosome, T, E> {
    fn transform(
        &self,
        engine_builder: GeneticEngineBuilder<C, T, E>,
        py_builder: &PyEngineBuilder,
        gene_type: crate::PyGeneType,
    ) -> GeneticEngineBuilder<C, T, E>
    where
        C: Chromosome + Clone,
        T: Clone,
        E: Epoch<Chromosome = C> + 'static;
}
