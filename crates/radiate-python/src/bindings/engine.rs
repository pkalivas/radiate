use crate::{
    EngineHandle, EpochHandle, PyGeneration, bindings::handles::EngineIterHandle, match_variant,
};
use pyo3::{PyRefMut, PyResult, Python, pyclass, pymethods};
use radiate::{Chromosome, Generation, GeneticEngine, Limit, RadiateResult};
use radiate_error::{radiate_py_bail, radiate_py_err};
use serde::Serialize;

const ENGINE_ALREADY_RUN_ERROR_STRING: &str =
    "Engine has already been run. Create a new engine instance to run again.";

const BUILD_ENGINE_WITH_LIMIT_ERROR_STRING: &str = "Engine must be built with at least one limit:
    engine = (
        rd.Engine.int(5)
        .fitness(my_fitness_fn)
        .limit(rd.Limit.generation(100)) # <- Must have at least one limit
    )";

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
                    .ok_or_else(|| radiate_py_err!(ENGINE_ALREADY_RUN_ERROR_STRING))?;

                if self.limits.is_empty() {
                    radiate_py_bail!(BUILD_ENGINE_WITH_LIMIT_ERROR_STRING);
                }

                self.iter = Some(engine.into_iter_handle(self.limits.clone()));
            }

            Ok(self
                .iter
                .as_mut()
                .and_then(|it| it.next_epoch().map(PyGeneration::new)))
        })
    }

    pub fn run(&mut self, py: Python, log: bool, ui: bool) -> PyResult<PyGeneration> {
        let handle = self
            .engine
            .take()
            .ok_or_else(|| radiate_py_err!(ENGINE_ALREADY_RUN_ERROR_STRING))?;

        if self.limits.is_empty() {
            radiate_py_bail!(BUILD_ENGINE_WITH_LIMIT_ERROR_STRING);
        }

        py.detach(|| {
            let limits = self.limits.clone();
            let epoch = match_variant!(EngineHandle, handle, engine => run_engine(engine, limits, log, ui)?.into());
            Ok(PyGeneration::new(epoch))
        })
    }
}

fn run_engine<C, T>(
    engine: GeneticEngine<C, T>,
    limits: Vec<Limit>,
    log: bool,
    ui: bool,
) -> RadiateResult<Generation<C, T>>
where
    C: Chromosome + Clone + Serialize + 'static,
    T: Clone + Send + Sync + Serialize + 'static,
    Generation<C, T>: Serialize + Into<EpochHandle>,
{
    if ui {
        radiate::ui(engine).iter().logging().limit(limits).last()
    } else {
        engine
            .iter()
            .chain_if(log, |eng| eng.logging())
            .limit(limits)
            .last()
    }
}
