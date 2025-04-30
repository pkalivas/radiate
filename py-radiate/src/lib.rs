use ::radiate_py::PyFloatGene;
use pyo3::prelude::*;

#[pymodule]
fn radiate(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PyFloatGene>()?;
    // Register your functions and classes here
    Ok(())
}
