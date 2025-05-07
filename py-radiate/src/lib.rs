use pyo3::prelude::*;

use radiate_python::{
    PyEngine,
    PyEngineBuilder,
    PyEngineParam,
    PyFloatCodex,
    PyRandomProvider,
    PySelector, // removeme,
};

#[pymodule]
fn radiate(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyEngine>()?;
    m.add_class::<PySelector>()?;
    m.add_class::<PyEngineParam>()?;
    m.add_class::<PyRandomProvider>()?;
    m.add_class::<PyFloatCodex>()?;
    m.add_class::<PyEngineBuilder>()?;

    Ok(())
}
