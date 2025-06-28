use std::fmt::Display;

use crate::conversion::Wrap;
use pyo3::{FromPyObject, pyclass, types::PyAnyMethods};
use radiate::Executor;

const WORKER_POOL: &str = "worker_pool";
const SERIAL: &str = "serial";

#[pyclass]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PyExecutor {
    WorkerPool,
    Serial,
}

#[pyo3::pymethods]
impl PyExecutor {
    pub fn __repr__(&self) -> String {
        match self {
            PyExecutor::WorkerPool => WORKER_POOL.into(),
            PyExecutor::Serial => SERIAL.into(),
        }
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }

    #[staticmethod]
    pub fn worker_pool() -> Self {
        PyExecutor::WorkerPool
    }

    #[staticmethod]
    pub fn serial() -> Self {
        PyExecutor::Serial
    }
}

impl Display for PyExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PyExecutor::WorkerPool => write!(f, "{}", WORKER_POOL),
            PyExecutor::Serial => write!(f, "{}", SERIAL),
        }
    }
}

impl<'py> FromPyObject<'py> for Wrap<Executor> {
    fn extract_bound(ob: &pyo3::Bound<'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let exec = ob.extract::<PyExecutor>()?;
        let executor = match exec {
            PyExecutor::WorkerPool => Executor::WorkerPool,
            PyExecutor::Serial => Executor::default(),
        };

        Ok(Wrap(executor))
    }
}
