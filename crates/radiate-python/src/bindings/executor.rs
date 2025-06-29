use std::fmt::Display;

use crate::conversion::Wrap;
use pyo3::{FromPyObject, pyclass, types::PyAnyMethods};
use radiate::Executor;

const WORKER_POOL: &str = "worker_pool";
const FIXED_SIZED_WORKER_POOL: &str = "fixed_sized_worker_pool";
const SERIAL: &str = "serial";

#[pyclass]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PyExecutorKind {
    WorkerPool,
    FixedSizedWorkerPool,
    Serial,
}

#[pyclass]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyExecutor {
    pub kind: PyExecutorKind,
    pub num_workers: Option<usize>,
}

#[pyo3::pymethods]
impl PyExecutor {
    pub fn __repr__(&self) -> String {
        match self.kind {
            PyExecutorKind::WorkerPool => WORKER_POOL.into(),
            PyExecutorKind::FixedSizedWorkerPool => {
                format!("{}({})", FIXED_SIZED_WORKER_POOL, self.num_workers.unwrap())
            }
            PyExecutorKind::Serial => SERIAL.into(),
        }
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }

    #[staticmethod]
    pub fn worker_pool() -> Self {
        PyExecutor {
            kind: PyExecutorKind::WorkerPool,
            num_workers: None,
        }
    }

    #[staticmethod]
    pub fn serial() -> Self {
        PyExecutor {
            kind: PyExecutorKind::Serial,
            num_workers: None,
        }
    }

    #[staticmethod]
    pub fn fixed_sized_worker_pool(num_workers: usize) -> Self {
        PyExecutor {
            kind: PyExecutorKind::FixedSizedWorkerPool,
            num_workers: Some(num_workers),
        }
    }
}

impl Display for PyExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            PyExecutorKind::WorkerPool => write!(f, "{}", WORKER_POOL),
            PyExecutorKind::FixedSizedWorkerPool => {
                write!(
                    f,
                    "{}({})",
                    FIXED_SIZED_WORKER_POOL,
                    self.num_workers.unwrap()
                )
            }
            PyExecutorKind::Serial => write!(f, "{}", SERIAL),
        }
    }
}

impl<'py> FromPyObject<'py> for Wrap<Executor> {
    fn extract_bound(ob: &pyo3::Bound<'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let exec = ob.extract::<PyExecutor>()?;
        let executor = match exec.kind {
            PyExecutorKind::WorkerPool => Executor::WorkerPool,
            PyExecutorKind::FixedSizedWorkerPool => {
                Executor::FixedSizedWorkerPool(exec.num_workers.unwrap())
            }
            PyExecutorKind::Serial => Executor::default(),
        };

        Ok(Wrap(executor))
    }
}
