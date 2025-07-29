use crate::{EngineHandle, EpochHandle, InputTransform, PyEngineInput, PyGeneration};
use pyo3::{PyResult, pyclass, pymethods};
use radiate::{Chromosome, Engine, EngineIteratorExt, Generation, GeneticEngine, Limit, Objective};
use tracing::info;

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
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Engine has already been run",
            ));
        }

        let engine = self.engine.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Engine has already been run")
        })?;

        let limits = limits.transform();

        if limits.is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "At least one limit must be specified",
            ));
        }

        Ok(PyGeneration::new(match engine {
            EngineHandle::Int(eng) => EpochHandle::Int(run_engine(eng, limits, log)),
            EngineHandle::Float(eng) => EpochHandle::Float(run_engine(eng, limits, log)),
            EngineHandle::Char(eng) => EpochHandle::Char(run_engine(eng, limits, log)),
            EngineHandle::Bit(eng) => EpochHandle::Bit(run_engine(eng, limits, log)),
            EngineHandle::Permutation(eng) => {
                EpochHandle::Permutation(run_engine(eng, limits, log))
            }
            EngineHandle::Graph(eng) => EpochHandle::Graph(run_engine(eng, limits, log)),
            EngineHandle::Tree(eng) => EpochHandle::Tree(run_engine(eng, limits, log)),
        }))
    }

    pub fn next(&mut self) -> PyResult<PyGeneration> {
        if self.engine.is_none() {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Engine has already been run",
            ));
        }

        let engine = self.engine.as_mut().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Engine has already been run")
        })?;

        Ok(PyGeneration::new(match engine {
            EngineHandle::Int(eng) => EpochHandle::Int(eng.next()),
            EngineHandle::Float(eng) => EpochHandle::Float(eng.next()),
            EngineHandle::Char(eng) => EpochHandle::Char(eng.next()),
            EngineHandle::Bit(eng) => EpochHandle::Bit(eng.next()),
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
    engine
        .iter()
        .inspect(move |epoch| {
            if log {
                match epoch.objective() {
                    Objective::Single(_) => {
                        info!(
                            "Epoch {:<4} | Score: {:>8.4} | Time: {:>5.2?}",
                            epoch.index(),
                            epoch.score().as_f32(),
                            epoch.time()
                        );
                    }
                    Objective::Multi(_) => {
                        info!(
                            "Epoch {:<4} | Front Size: {:>4} | Time: {:>5.2?}",
                            epoch.index(),
                            epoch.front().map_or(0, |front| front.values().len()),
                            epoch.time()
                        );
                    }
                }
            }
        })
        .limit(limits)
        .last()
        .expect("No generation found that meets the limits")
}
