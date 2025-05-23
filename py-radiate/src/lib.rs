use pyo3::prelude::*;
use radiate_python::{
    PyBitCodex, PyBitEngine, PyCharCodex, PyCharEngine, PyEngineBuilder, PyEngineParam,
    PyFloatCodex, PyFloatEngine, PyGeneration, PyIntCodex, PyIntEngine, PyRandomProvider,
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

    m.add_class::<PyCharCodex>()?;
    m.add_class::<PyCharEngine>()?;

    m.add_class::<PyBitCodex>()?;
    m.add_class::<PyBitEngine>()?;

    m.add_class::<PyGeneration>()?;

    Ok(())
}
