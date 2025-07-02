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

        let result = match engine {
            EngineHandle::Int(eng) => {
                let output = run_single_objective_engine(eng, limits, log);
                EpochHandle::Int(output)
            }
            EngineHandle::Float(eng) => {
                let output = run_single_objective_engine(eng, limits, log);
                EpochHandle::Float(output)
            }
            EngineHandle::Char(eng) => {
                let output = run_single_objective_engine(eng, limits, log);
                EpochHandle::Char(output)
            }
            EngineHandle::Bit(eng) => {
                let output = run_single_objective_engine(eng, limits, log);
                EpochHandle::Bit(output)
            }
            EngineHandle::Permutation(eng) => {
                let output = run_single_objective_engine(eng, limits, log);
                EpochHandle::Permutation(output)
            }
            EngineHandle::Graph(eng) => {
                let output = run_single_objective_engine(eng, limits, log);
                EpochHandle::Graph(output)
            }
            EngineHandle::Tree(eng) => {
                let output = run_single_objective_engine(eng, limits, log);
                EpochHandle::Tree(output)
            }
        };

        Ok(PyGeneration::new(result))
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

fn run_single_objective_engine<C, T>(
    engine: GeneticEngine<C, T>,
    limits: Vec<Limit>,
    log: bool,
) -> Generation<C, T>
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
