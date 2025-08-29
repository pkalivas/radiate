use crate::{EngineHandle, EpochHandle, InputTransform, PyEngineInput, PyGeneration};
use pyo3::{
    PyResult,
    exceptions::{PyRuntimeError, PyValueError},
    pyclass, pymethods,
};
use radiate::{Chromosome, Engine, EngineIteratorExt, Generation, GeneticEngine, Limit};

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
        if self.engine.is_none() {
            return Err(PyRuntimeError::new_err("Engine has already been run"));
        }

        let engine = self
            .engine
            .take()
            .ok_or_else(|| PyRuntimeError::new_err("Engine has already been run"))?;

        let limits = limits.transform();

        if limits.is_empty() {
            return Err(PyValueError::new_err(
                "At least one limit must be specified",
            ));
        }

        Ok(PyGeneration::new(match engine {
            EngineHandle::Int(eng) => EpochHandle::Int(run_engine(eng, limits, log)),
            EngineHandle::Float(eng) => EpochHandle::Float(run_engine(eng, limits, log)),
            EngineHandle::Char(eng) => EpochHandle::Char(run_engine(eng, limits, log)),
            EngineHandle::Bit(eng) => EpochHandle::Bit(run_engine(eng, limits, log)),
            EngineHandle::Any(eng) => EpochHandle::Any(run_engine(eng, limits, log)),
            EngineHandle::Permutation(eng) => {
                EpochHandle::Permutation(run_engine(eng, limits, log))
            }
            EngineHandle::Graph(eng) => EpochHandle::Graph(run_engine(eng, limits, log)),
            EngineHandle::Tree(eng) => EpochHandle::Tree(run_engine(eng, limits, log)),
        }))
    }

    pub fn next(&mut self) -> PyResult<PyGeneration> {
        if self.engine.is_none() {
            return Err(PyRuntimeError::new_err("Engine has already been run"));
        }

        let engine = self
            .engine
            .as_mut()
            .ok_or_else(|| PyRuntimeError::new_err("Engine has already been run"))?;

        Ok(PyGeneration::new(match engine {
            EngineHandle::Int(eng) => EpochHandle::Int(eng.next()),
            EngineHandle::Float(eng) => EpochHandle::Float(eng.next()),
            EngineHandle::Char(eng) => EpochHandle::Char(eng.next()),
            EngineHandle::Bit(eng) => EpochHandle::Bit(eng.next()),
            EngineHandle::Any(eng) => EpochHandle::Any(eng.next()),
            EngineHandle::Permutation(eng) => EpochHandle::Permutation(eng.next()),
            EngineHandle::Graph(eng) => EpochHandle::Graph(eng.next()),
            EngineHandle::Tree(eng) => EpochHandle::Tree(eng.next()),
        }))
    }
}

fn run_engine<C, T>(engine: GeneticEngine<C, T>, limits: Vec<Limit>, log: bool) -> Generation<C, T>
where
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    match log {
        true => engine.iter().logging().limit(limits),
        false => engine.iter().limit(limits),
    }
    .last()
    .expect("No generation found that meets the limits")
}
