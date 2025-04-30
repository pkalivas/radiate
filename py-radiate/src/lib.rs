use pyo3::prelude::*;

use radiate_python::PyFloatScalarEngine;

#[pymodule]
fn radiate(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyFloatScalarEngine>()?;
    Ok(())
}
