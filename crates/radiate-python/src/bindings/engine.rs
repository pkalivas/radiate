use crate::{
    EngineHandle, EpochHandle, InputTransform, PickleWriter, PyEngineInput, PyGeneration,
    bindings::handles::StepHandle, names,
};
use pyo3::{PyResult, Python, pyclass, pymethods};
use radiate::{
    Chromosome, Engine, EngineRuntime, EvolutionContext, Generation, GeneticEngine, JsonWriter,
    Limit,
};
use radiate_error::{radiate_py_bail, radiate_py_err};
use serde::Serialize;
use std::time::Duration;

const BUILD_ENGINE_WITH_LIMIT_ERROR_STRING: &str = "Engine must be built with at least one limit:
    engine = (
        rd.Engine.int(5)
        .fitness(my_fitness_fn)
        .limit(rd.Limit.generation(100)) # <- Must have at least one limit
    )";

#[pyclass(from_py_object)]
#[derive(Clone)]
pub enum PyEngineRunOption {
    Log(bool),
    Checkpoint(usize, String, String),
    Ui(Duration),
}

#[pymethods]
impl PyEngineRunOption {
    #[staticmethod]
    pub fn log(value: bool) -> Self {
        PyEngineRunOption::Log(value)
    }

    #[staticmethod]
    pub fn checkpoint(interval: usize, path: String, file_type: String) -> Self {
        PyEngineRunOption::Checkpoint(interval, path, file_type)
    }

    #[staticmethod]
    pub fn ui() -> Self {
        PyEngineRunOption::Ui(radiate::DEFAULT_RENDER_INTERVAL)
    }
}

#[pyclass(unsendable)]
pub struct PyEngine {
    engine: Option<EngineHandle>,
    iter: Option<StepHandle>,
    limits: Vec<Limit>,
}

impl PyEngine {
    pub fn new(limits: Vec<Limit>, engine: EngineHandle) -> Self {
        Self {
            engine: Some(engine),
            iter: None,
            limits,
        }
    }
}

#[pymethods]
impl PyEngine {
    pub fn run(
        &mut self,
        py: Python,
        limits: Vec<PyEngineInput>,
        options: Vec<PyEngineRunOption>,
    ) -> PyResult<PyGeneration> {
        use EngineHandle::*;
        let engine = self
            .engine
            .take()
            .ok_or_else(|| radiate_py_err!("Engine has already been run"))?;

        let limits = self
            .limits
            .clone()
            .into_iter()
            .chain(limits.into_iter().filter_map(|input| input.transform()))
            .collect::<Vec<_>>();

        if limits.is_empty() {
            radiate_py_bail!(BUILD_ENGINE_WITH_LIMIT_ERROR_STRING);
        }

        py.detach(|| {
            Ok(PyGeneration::new(match engine {
                UInt8(eng) => EpochHandle::UInt8(run_engine(eng, limits, options)?),
                UInt16(eng) => EpochHandle::UInt16(run_engine(eng, limits, options)?),
                UInt32(eng) => EpochHandle::UInt32(run_engine(eng, limits, options)?),
                UInt64(eng) => EpochHandle::UInt64(run_engine(eng, limits, options)?),
                Int8(eng) => EpochHandle::Int8(run_engine(eng, limits, options)?),
                Int16(eng) => EpochHandle::Int16(run_engine(eng, limits, options)?),
                Int32(eng) => EpochHandle::Int32(run_engine(eng, limits, options)?),
                Int64(eng) => EpochHandle::Int64(run_engine(eng, limits, options)?),
                Float32(eng) => EpochHandle::Float32(run_engine(eng, limits, options)?),
                Float64(eng) => EpochHandle::Float64(run_engine(eng, limits, options)?),
                Char(eng) => EpochHandle::Char(run_engine(eng, limits, options)?),
                Bit(eng) => EpochHandle::Bit(run_engine(eng, limits, options)?),
                Permutation(eng) => EpochHandle::Permutation(run_engine(eng, limits, options)?),
                Graph(eng) => EpochHandle::Graph(run_engine(eng, limits, options)?),
                Tree(eng) => EpochHandle::Tree(run_engine(eng, limits, options)?),
            }))
        })
    }

    pub fn step_next(&mut self, py: Python) -> PyResult<PyGeneration> {
        use StepHandle::*;

        if self.iter.is_none() {
            let engine = self
                .engine
                .take()
                .ok_or_else(|| radiate_py_err!("Engine has already been run"))?;

            if self.limits.is_empty() {
                radiate_py_bail!(BUILD_ENGINE_WITH_LIMIT_ERROR_STRING);
            }

            self.iter = Some(engine.into_step(self.limits.clone()));
        }
        py.detach(|| {
            let next = match self.iter.as_mut().unwrap() {
                UInt8(it) => it.next().map(EpochHandle::UInt8),
                UInt16(it) => it.next().map(EpochHandle::UInt16),
                UInt32(it) => it.next().map(EpochHandle::UInt32),
                UInt64(it) => it.next().map(EpochHandle::UInt64),
                Int8(it) => it.next().map(EpochHandle::Int8),
                Int16(it) => it.next().map(EpochHandle::Int16),
                Int32(it) => it.next().map(EpochHandle::Int32),
                Int64(it) => it.next().map(EpochHandle::Int64),
                Float32(it) => it.next().map(EpochHandle::Float32),
                Float64(it) => it.next().map(EpochHandle::Float64),
                Char(it) => it.next().map(EpochHandle::Char),
                Bit(it) => it.next().map(EpochHandle::Bit),
                Permutation(it) => it.next().map(EpochHandle::Permutation),
                Graph(it) => it.next().map(EpochHandle::Graph),
                Tree(it) => it.next().map(EpochHandle::Tree),
            };

            next.map(PyGeneration::new)
                .ok_or_else(|| radiate_py_err!("Engine has already completed all steps"))
        })
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

fn iter_engine<E, C, T>(
    engine: EngineRuntime<E>,
    limits: Vec<Limit>,
    options: Vec<PyEngineRunOption>,
) -> PyResult<Generation<C, T>>
where
    E: Engine<Epoch = Generation<C, T>, Ctx = EvolutionContext<C, T>> + 'static,
    E::Epoch: Serialize,
    C: Chromosome + Clone + Serialize + 'static,
    T: Clone + Send + Sync + Serialize + 'static,
{
    let log = get_log_option(&options);
    let checkpoint = get_checkpoint_option(&options);

    engine
        .chain_if(log.unwrap_or(false), |eng| eng.logging())
        .chain_if(checkpoint.is_some(), |eng| {
            let (interval, path, file_type) = checkpoint.unwrap();
            match file_type.as_str() {
                names::JSON_FILE_TYPE => eng.checkpoint_with(interval, path, Box::new(JsonWriter)),
                _ => eng.checkpoint_with(interval, path, Box::new(PickleWriter)),
            }
        })
        .limit(limits)
        .last()
        .map_err(|err| radiate_py_err!(format!("Engine failed during execution: {err}")))
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

fn get_checkpoint_option(options: &[PyEngineRunOption]) -> Option<(usize, String, String)> {
    options
        .iter()
        .find(|opt| matches!(opt, PyEngineRunOption::Checkpoint(_, _, _)))
        .and_then(|opt| {
            if let PyEngineRunOption::Checkpoint(interval, path, file_type) = opt {
                Some((*interval, path.clone(), file_type.clone()))
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
