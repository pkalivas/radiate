use pyo3::prelude::*;

use radiate_python::{
    PyFloatEngine,
    {
        PyAnyChromosome, PyAnyGene, PyBitChromosome, PyBitGene, PyCharChromosome, PyCharGene,
        PyFloatChromosome, PyFloatGene, PyIntChromosome, PyIntGene,
    },
};

#[pymodule]
fn radiate(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyFloatEngine>()?;

    m.add_class::<PyFloatGene>()?;
    m.add_class::<PyAnyGene>()?;
    m.add_class::<PyIntGene>()?;
    m.add_class::<PyBitGene>()?;
    m.add_class::<PyCharGene>()?;

    m.add_class::<PyFloatChromosome>()?;
    m.add_class::<PyIntChromosome>()?;
    m.add_class::<PyBitChromosome>()?;
    m.add_class::<PyCharChromosome>()?;
    m.add_class::<PyAnyChromosome>()?;

    Ok(())
}
