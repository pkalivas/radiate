use pyo3::prelude::*;

use radiate_python::PyEngine;

#[pymodule]
fn radiate(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyEngine>()?;

    Ok(())
}
