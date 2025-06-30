use crate::{EngineHandle, EpochHandle, PyEngineInput, PyGeneration};
use pyo3::{PyResult, pyclass, pymethods};
use radiate::{
    Chromosome, Engine, Epoch, Generation, GeneticEngine, Limit, Objective, Optimize,
    ParetoGeneration, log_ctx,
};

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
            EngineHandle::IntMulti(eng) => {
                let output = run_multi_objective_engine(eng, limits, log);
                EpochHandle::IntMulti(output)
            }
            EngineHandle::FloatMulti(eng) => {
                let output = run_multi_objective_engine(eng, limits, log);
                EpochHandle::FloatMulti(output)
            }
            EngineHandle::GraphRegression(eng) => {
                let output = run_single_objective_engine(eng, limits, log);
                EpochHandle::GraphRegression(output)
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
            EngineHandle::IntMulti(eng) => EpochHandle::IntMulti(eng.next()),
            EngineHandle::FloatMulti(eng) => EpochHandle::FloatMulti(eng.next()),
            EngineHandle::GraphRegression(eng) => EpochHandle::GraphRegression(eng.next()),
        };

        Ok(PyGeneration::new(result))
    }
}

fn run_single_objective_engine<C, T>(
    engine: GeneticEngine<C, T, Generation<C, T>>,
    limits: Vec<Limit>,
    log: bool,
) -> Generation<C, T>
where
    C: Chromosome + Clone,
    T: Clone + Send + Sync + 'static,
{
    engine
        .iter()
        .inspect(|epoch| {
            if log {
                log_ctx!(epoch);
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

fn run_multi_objective_engine<C, T>(
    engine: GeneticEngine<C, T, ParetoGeneration<C>>,
    limits: Vec<Limit>,
    log: bool,
) -> ParetoGeneration<C>
where
    C: Chromosome + Clone,
    T: Clone + Send + Sync + 'static,
{
    engine
        .iter()
        .inspect(|epoch| {
            if log {
                println!("{}", epoch.index());
            }
        })
        .skip_while(|epoch| {
            limits.iter().all(|limit| match limit {
                Limit::Generation(lim) => epoch.index() < *lim,
                Limit::Score(_) => false,
                Limit::Seconds(val) => return epoch.seconds() < *val,
            })
        })
        .take(1)
        .last()
        .expect("No generation found that meets the limits")
}
