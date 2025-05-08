use pyo3::prelude::*;

use radiate_python::{
    PyEngineBuilder, PyEngineParam, PyFloatCodex, PyFloatEngine, PyIntCodex, PyIntEngine,
    PyRandomProvider,
};

#[pymodule]
fn radiate(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyEngineParam>()?;
    m.add_class::<PyEngineBuilder>()?;

    m.add_class::<PyRandomProvider>()?;

    m.add_class::<PyFloatCodex>()?;
    m.add_class::<PyIntCodex>()?;

    m.add_class::<PyFloatEngine>()?;
    m.add_class::<PyIntEngine>()?;

    Ok(())
}
