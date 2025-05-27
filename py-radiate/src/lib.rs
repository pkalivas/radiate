use pyo3::prelude::*;
use radiate_python::{
    PyBitCodec, PyBitEngine, PyCharCodec, PyCharEngine, PyEngineBuilder, PyEngineParam,
    PyFloatCodec, PyFloatEngine, PyGeneration, PyIntCodec, PyIntEngine, PyRandomProvider,
};

#[pymodule]
fn radiate(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyEngineParam>()?;
    m.add_class::<PyEngineBuilder>()?;

    m.add_class::<PyRandomProvider>()?;

    m.add_class::<PyFloatCodec>()?;
    m.add_class::<PyIntCodec>()?;

    m.add_class::<PyFloatEngine>()?;
    m.add_class::<PyIntEngine>()?;

    m.add_class::<PyCharCodec>()?;
    m.add_class::<PyCharEngine>()?;

    m.add_class::<PyBitCodec>()?;
    m.add_class::<PyBitEngine>()?;

    m.add_class::<PyGeneration>()?;

    Ok(())
}
