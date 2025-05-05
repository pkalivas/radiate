use pyo3::prelude::*;

use radiate_python::{PyEngine, PyEngineParam, PyRandomProvider, PySelector};

#[pymodule]
fn radiate(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyEngine>()?;
    m.add_class::<PySelector>()?;
    m.add_class::<PyEngineParam>()?;
    m.add_class::<PyRandomProvider>()?;

    Ok(())
}
