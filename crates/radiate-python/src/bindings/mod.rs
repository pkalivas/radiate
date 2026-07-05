mod builder;
mod codec;
mod converters;
mod datatype;
mod engine;
mod epoch;
mod expr;
mod fitness;
mod front;
mod functions;
mod genome;
mod gp;
mod handles;
mod inputs;
mod metric;
mod subscriber;

pub use builder::*;
pub use codec::{
    PyBitCodec, PyCharCodec, PyCodec, PyFloatCodec, PyGraphCodec, PyIntCodec, PyPermutationCodec,
    PyTreeCodec,
};
pub use converters::InputTransform;
pub use datatype::{_get_dtype_max, _get_dtype_min, dtype, dtype_from_str};
pub use engine::{PyEngine, PyEngineControl, PyEngineRunOption};
pub use epoch::PyGeneration;
pub use expr::PyExpr;
pub use fitness::{PyFitnessFn, PyFitnessInner, PyNoveltySearch};
pub use front::{PyFront, PyFrontValue};
pub use functions::*;
pub use genome::*;
pub use gp::{
    _activation_ops, _all_ops, _create_op, _edge_ops, PyAccuracy, PyGraph, PyOp, PyTree,
    py_accuracy,
};
pub use handles::{EngineBuilderHandle, EngineHandle, EpochHandle};
pub use inputs::{PyEngineInput, PyEngineInputType};
pub use metric::{PyMetric, PyMetricSet};
use pyo3::{Py, Python, sync::PyOnceLock, types::PyModule};
pub use subscriber::{PyEngineEvent, PySubscriber};

static RADIATE: PyOnceLock<Py<PyModule>> = PyOnceLock::new();
static NUMPY: PyOnceLock<Py<PyModule>> = PyOnceLock::new();

pub fn radiate(py: Python<'_>) -> &Py<PyModule> {
    RADIATE.get_or_init(py, || py.import("radiate").unwrap().unbind())
}

pub fn numpy(py: Python<'_>) -> &Py<PyModule> {
    NUMPY.get_or_init(py, || py.import("numpy").unwrap().unbind())
}
