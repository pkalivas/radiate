use crate::{EngineHandle, EpochHandle, PyEngineInput, PyGeneration};
use pyo3::{PyResult, pyclass, pymethods};
use radiate::{Chromosome, Engine, Generation, GeneticEngine, Limit, Objective, Optimize};
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

        let limits = limits
            .into_iter()
            .filter_map(|input| input.into())
            .collect::<Vec<Limit>>();

        let engine = self.engine.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Engine has already been run")
        })?;

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

        let result = match engine {
            EngineHandle::Int(eng) => EpochHandle::Int(eng.next()),
            EngineHandle::Float(eng) => EpochHandle::Float(eng.next()),
            EngineHandle::Char(eng) => EpochHandle::Char(eng.next()),
            EngineHandle::Bit(eng) => EpochHandle::Bit(eng.next()),
            EngineHandle::Permutation(eng) => EpochHandle::Permutation(eng.next()),
            EngineHandle::Graph(eng) => EpochHandle::Graph(eng.next()),
            EngineHandle::Tree(eng) => EpochHandle::Tree(eng.next()),
        };

        Ok(PyGeneration::new(result))
    }
}

fn run_engine<C, T>(engine: GeneticEngine<C, T>, limits: Vec<Limit>, log: bool) -> Generation<C, T>
where
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    engine
        .iter()
        .inspect(|epoch| {
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
        .skip_while(|epoch| {
            limits.iter().all(|limit| match limit {
                Limit::Generation(lim) => epoch.index() < *lim,
                Limit::Score(lim) => match epoch.objective() {
                    Objective::Single(opt) => match opt {
                        Optimize::Minimize => epoch.score().as_f32() > *lim,
                        Optimize::Maximize => epoch.score().as_f32() < *lim,
                    },
                    Objective::Multi(_) => false,
                },
                Limit::Seconds(val) => return epoch.seconds() < *val,
            })
        })
        .take(1)
        .last()
        .expect("No generation found that meets the limits")
}
