use pyo3::prelude::*;

use radiate_python::{PyChromosome, PyFloatEngine, PyFloatGene, PyGene};

#[pymodule]
fn radiate(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyFloatEngine>()?;
    m.add_class::<PyFloatGene>()?;
    m.add_class::<PyGene>()?;
    m.add_class::<PyChromosome>()?;

    Ok(())
}
