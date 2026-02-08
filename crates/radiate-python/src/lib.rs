mod any;
mod bindings;
mod evaluator;
mod events;
mod names;
mod object;
mod problem;
mod random;

pub use any::*;
pub use bindings::*;
pub use evaluator::FreeThreadPyEvaluator;
pub use object::*;
pub use problem::PyProblem;
pub use pyo3::{PyResult, Python, exceptions::PyRuntimeError};
pub use random::PyRandomProvider;

pub mod prelude {
    pub use super::{IntoPyAnyObject, PyAnyObject, PyProblem};
    pub use crate::{
        PyAnyCodec, PyBitCodec, PyCharCodec, PyFloatCodec, PyGeneType, PyGraphCodec, PyIntCodec,
        Wrap,
    };
    pub use pyo3::prelude::*;
    pub use pyo3::types::PyAny;
    pub use pyo3::{
        Bound, IntoPyObjectExt, Py, PyErr, PyResult, Python, pyclass, pymethods,
        types::{PyAnyMethods, PyDict, PyDictMethods, PyList, PyString, PyTuple},
    };
}

// Simple passthrough to the core logging
pub fn init_logging() {
    radiate::init_logging();
}
