use pyo3::prelude::*;

use radiate_python::{PyEngine, PyEngineParam, PySelector};

#[pymodule]
fn radiate(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyEngine>()?;
    m.add_class::<PySelector>()?;
    m.add_class::<PyEngineParam>()?;

    Ok(())
}
