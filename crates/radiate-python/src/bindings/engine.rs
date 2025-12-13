use crate::{EngineHandle, EpochHandle, InputTransform, PyEngineInput, PyGeneration};
use pyo3::{PyResult, pyclass, pymethods};
use radiate::{
    Chromosome, Engine, EngineIteratorExt, Generation, GeneticEngine, Limit, radiate_err,
};
use radiate_error::{radiate_py_bail, radiate_py_err};
use serde::Serialize;
use std::time::Duration;

#[pyclass]
#[derive(Clone)]
pub enum PyEngineRunOption {
    Log(bool),
    Checkpoint(usize, String),
    Ui(Duration),
}

#[pymethods]
impl PyEngineRunOption {
    #[staticmethod]
    pub fn log(value: bool) -> Self {
        PyEngineRunOption::Log(value)
    }

    #[staticmethod]
    pub fn checkpoint(interval: usize, path: String) -> Self {
        PyEngineRunOption::Checkpoint(interval, path)
    }

    #[staticmethod]
    pub fn ui() -> Self {
        PyEngineRunOption::Ui(radiate::DEFAULT_RENDER_INTERVAL)
    }
}

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
    pub fn run(
        &mut self,
        limits: Vec<PyEngineInput>,
        options: Vec<PyEngineRunOption>,
    ) -> PyResult<PyGeneration> {
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
            Int(eng) => EpochHandle::Int(run_engine(eng, limits, options)?),
            Float(eng) => EpochHandle::Float(run_engine(eng, limits, options)?),
            Char(eng) => EpochHandle::Char(run_engine(eng, limits, options)?),
            Bit(eng) => EpochHandle::Bit(run_engine(eng, limits, options)?),
            Any(eng) => EpochHandle::Any(run_engine(eng, limits, options)?),
            Permutation(eng) => EpochHandle::Permutation(run_engine(eng, limits, options)?),
            Graph(eng) => EpochHandle::Graph(run_engine(eng, limits, options)?),
            Tree(eng) => EpochHandle::Tree(run_engine(eng, limits, options)?),
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
    options: Vec<PyEngineRunOption>,
) -> PyResult<Generation<C, T>>
where
    C: Chromosome + Clone + Serialize + 'static,
    T: Clone + Send + Sync + Serialize + 'static,
{
    let ui_interval = get_ui_option(&options);
    if let Some(interval) = ui_interval {
        iter_engine(radiate::ui((engine, interval)).iter(), limits, options)
    } else {
        iter_engine(engine.iter(), limits, options)
    }
}

fn iter_engine<C, T>(
    engine: impl Iterator<Item = Generation<C, T>> + 'static,
    limits: Vec<Limit>,
    options: Vec<PyEngineRunOption>,
) -> PyResult<Generation<C, T>>
where
    C: Chromosome + Clone + Serialize + 'static,
    T: Clone + Send + Sync + Serialize + 'static,
{
    let log = get_log_option(&options);
    let checkpoint = get_checkpoint_option(&options);

    engine
        .chain_if(log.unwrap_or(false), |eng| eng.logging())
        .chain_if(checkpoint.is_some(), |eng| {
            let (interval, path) = checkpoint.unwrap();
            eng.checkpoint(interval, path)
        })
        .limit(limits)
        .last()
        .ok_or_else(|| radiate_err!(Python: "Failed to run engine and obtain final generation"))
}

fn get_log_option(options: &[PyEngineRunOption]) -> Option<bool> {
    let ui = get_ui_option(options);
    let log = options
        .iter()
        .find(|opt| matches!(opt, PyEngineRunOption::Log(_)))
        .and_then(|opt| {
            if let PyEngineRunOption::Log(val) = opt {
                Some(*val)
            } else {
                None
            }
        });

    if ui.is_some() { Some(false) } else { log }
}

fn get_checkpoint_option(options: &[PyEngineRunOption]) -> Option<(usize, String)> {
    options
        .iter()
        .find(|opt| matches!(opt, PyEngineRunOption::Checkpoint(_, _)))
        .and_then(|opt| {
            if let PyEngineRunOption::Checkpoint(interval, path) = opt {
                Some((*interval, path.clone()))
            } else {
                None
            }
        })
}

fn get_ui_option(options: &[PyEngineRunOption]) -> Option<Duration> {
    options
        .iter()
        .find(|opt| matches!(opt, PyEngineRunOption::Ui(_)))
        .and_then(|opt| {
            if let PyEngineRunOption::Ui(interval) = opt {
                Some(*interval)
            } else {
                None
            }
        })
}
