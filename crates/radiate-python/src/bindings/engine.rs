use crate::{EngineHandle, EpochHandle, InputTransform, PyEngineInput, PyGeneration};
use pyo3::{PyResult, pyclass, pymethods};
use radiate::{
    Chromosome, Engine, EngineIteratorExt, Generation, GeneticEngine, Limit, radiate_err,
};
use radiate_error::{radiate_py_bail, radiate_py_err};

#[pyclass(unsendable)]
pub struct PyEngine {
    engine: Option<EngineHandle>,
}

impl PyEngine {
    pub fn new(engine: EngineHandle) -> Self {
        Self {
            engine: Some(engine),
        }
    }
}

#[pymethods]
impl PyEngine {
    pub fn run(&mut self, limits: Vec<PyEngineInput>, log: bool) -> PyResult<PyGeneration> {
        use EngineHandle::*;
        let engine = self
            .engine
            .take()
            .ok_or_else(|| radiate_py_err!("Engine has already been run"))?;

        let limits = limits.transform();

        if limits.is_empty() {
            radiate_py_bail!("At least one limit must be provided");
        }

        Ok(PyGeneration::new(match engine {
            Int(eng) => EpochHandle::Int(run_engine(eng, limits, log)?),
            Float(eng) => EpochHandle::Float(run_engine(eng, limits, log)?),
            Char(eng) => EpochHandle::Char(run_engine(eng, limits, log)?),
            Bit(eng) => EpochHandle::Bit(run_engine(eng, limits, log)?),
            Any(eng) => EpochHandle::Any(run_engine(eng, limits, log)?),
            Permutation(eng) => EpochHandle::Permutation(run_engine(eng, limits, log)?),
            Graph(eng) => EpochHandle::Graph(run_engine(eng, limits, log)?),
            Tree(eng) => EpochHandle::Tree(run_engine(eng, limits, log)?),
        }))
    }

    pub fn next(&mut self) -> PyResult<PyGeneration> {
        use EngineHandle::*;
        let engine = self
            .engine
            .as_mut()
            .ok_or_else(|| radiate_py_err!("Engine has already been run"))?;

        Ok(PyGeneration::new(match engine {
            Int(eng) => EpochHandle::Int(eng.next()?),
            Float(eng) => EpochHandle::Float(eng.next()?),
            Char(eng) => EpochHandle::Char(eng.next()?),
            Bit(eng) => EpochHandle::Bit(eng.next()?),
            Any(eng) => EpochHandle::Any(eng.next()?),
            Permutation(eng) => EpochHandle::Permutation(eng.next()?),
            Graph(eng) => EpochHandle::Graph(eng.next()?),
            Tree(eng) => EpochHandle::Tree(eng.next()?),
        }))
    }
}

fn run_engine<C, T>(
    engine: GeneticEngine<C, T>,
    limits: Vec<Limit>,
    log: bool,
) -> PyResult<Generation<C, T>>
where
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    match log {
        true => engine.iter().logging().limit(limits),
        false => engine.iter().limit(limits),
    }
    .last()
    .ok_or_else(|| radiate_err!(Python: "Failed to run engine and obtain final generation"))
}
