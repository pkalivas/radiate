use pyo3::prelude::*;
use radiate_py::PyFloatGene;

#[pymodule]
fn radiate(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PyFloatGene>()?;
    // Register your functions and classes here
    Ok(())
}
