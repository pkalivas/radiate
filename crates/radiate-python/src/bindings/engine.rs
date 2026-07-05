use crate::{
    EngineHandle, EpochHandle, InputTransform, PyCheckpointWriter, PyEngineInput, PyGeneration,
    bindings::handles::EngineIterHandle, match_variant,
};
use pyo3::{PyRefMut, PyResult, Python, pyclass, pymethods};
use radiate::{
    Chromosome, Engine, EngineControl, EngineRuntime, EvolutionContext, Generation, GeneticEngine,
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

#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct PyEngineControl {
    control: EngineControl,
}

#[pymethods]
impl PyEngineControl {
    pub fn pause(&mut self) {
        self.control.set_paused(true);
    }

    pub fn resume(&mut self) {
        self.control.set_paused(false);
    }

    pub fn stop(&mut self) {
        self.control.stop();
    }

    pub fn is_paused(&self) -> bool {
        self.control.is_paused()
    }

    pub fn is_stopped(&self) -> bool {
        self.control.is_stopped()
    }

    pub fn step_once(&mut self) {
        self.control.step_once();
    }
}

#[pyclass(unsendable)]
pub struct PyEngine {
    engine: Option<EngineHandle>,
    iter: Option<EngineIterHandle>,
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
    pub fn __iter__(slf: PyRefMut<Self>) -> PyRefMut<Self> {
        slf
    }

    pub fn __next__(&mut self, py: Python) -> PyResult<Option<PyGeneration>> {
        py.detach(|| {
            if self.iter.is_none() {
                let engine = self
                    .engine
                    .take()
                    .ok_or_else(|| radiate_py_err!("Engine has already been run"))?;

                if self.limits.is_empty() {
                    radiate_py_bail!(BUILD_ENGINE_WITH_LIMIT_ERROR_STRING);
                }

                self.iter = Some(engine.into_iter_handle(self.limits.clone()));
            }

            Ok(self
                .iter
                .as_mut()
                .unwrap()
                .next_epoch()
                .map(PyGeneration::new))
        })
    }

    pub fn run(
        &mut self,
        py: Python,
        limits: Vec<PyEngineInput>,
        options: Vec<PyEngineRunOption>,
    ) -> PyResult<PyGeneration> {
        let engine_handle = self
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
            let epoch_handle = match_variant!(EngineHandle, engine_handle, engine => EpochHandle::from(run_engine(engine, limits, options)?));
            Ok(PyGeneration::new(epoch_handle))
        })
    }

    pub fn control(&mut self) -> PyResult<PyEngineControl> {
        match self.engine {
            Some(ref mut engine) => {
                let control = match_variant!(EngineHandle, engine, engine => engine.control());
                Ok(PyEngineControl { control })
            }
            None => Err(radiate_py_err!("Engine has already been run")),
        }
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
    Generation<C, T>: Serialize + Into<EpochHandle>,
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
    E::Epoch: Serialize + Into<EpochHandle>,
    C: Chromosome + Clone + Serialize + 'static,
    T: Clone + Send + Sync + Serialize + 'static,
{
    let log = get_log_option(&options);
    let checkpoint = get_checkpoint_option(&options);

    engine
        .chain_if(log.unwrap_or(false), |eng| eng.logging())
        .chain_if(checkpoint.is_some(), |eng| {
            let (interval, path, file_type) = checkpoint.unwrap();
            eng.checkpoint_with(interval, path, Box::new(PyCheckpointWriter(file_type)))
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
