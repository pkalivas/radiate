use crate::{
    AnyValue,
    conversion::{ObjectValue, Wrap},
};
use pyo3::{Py, PyAny, PyObject, Python};
use radiate::Score;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ThreadSafePythonFn {
    func: Arc<Py<PyAny>>,
}

impl ThreadSafePythonFn {
    pub fn new(func: PyObject) -> Self {
        Self {
            func: Arc::new(func.into()),
        }
    }

    pub fn call<'py>(&self, outer: Python<'py>, input: ObjectValue) -> Score {
        let any_value = self
            .func
            .call1(outer, (input.inner,))
            .expect("Python call failed");

        let av = any_value
            .extract::<Wrap<AnyValue<'_>>>(outer)
            .expect("Python function must return a valid value")
            .0;

        match av {
            AnyValue::Float32(score) => Score::from(score),
            AnyValue::Float64(score) => Score::from(score as f32),
            AnyValue::Int32(score) => Score::from(score as f32),
            AnyValue::Int64(score) => Score::from(score as f32),
            AnyValue::Int128(score) => Score::from(score as f32),
            AnyValue::Int16(score) => Score::from(score as f32),
            AnyValue::Int8(score) => Score::from(score as f32),
            AnyValue::Boolean(score) => Score::from(if score { 1.0 } else { 0.0 }),
            AnyValue::Null => Score::from(0.0),
            _ => panic!("Fitness function must return a number"),
        }
    }
}
