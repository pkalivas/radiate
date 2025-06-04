use pyo3::prelude::*;
use radiate_python::{
    PyAlterer, PyBitCodec, PyCharCodec, PyChromosome, PyDiversity, PyEngineBuilder, PyFloatCodec,
    PyGene, PyGeneType, PyGeneration, PyGenotype, PyIntCodec, PyLimit, PyObjective, PyPhenotype,
    PyPopulation, PyRandomProvider, PySelector, PySubscriber,
};

#[pymodule]
fn radiate(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyRandomProvider>()?;

    m.add_class::<PyGeneType>()?;
    m.add_class::<PyGene>()?;
    m.add_class::<PyPhenotype>()?;
    m.add_class::<PyChromosome>()?;
    m.add_class::<PyGenotype>()?;
    m.add_class::<PyPopulation>()?;

    m.add_class::<PyFloatCodec>()?;
    m.add_class::<PyIntCodec>()?;
    m.add_class::<PyCharCodec>()?;
    m.add_class::<PyBitCodec>()?;

    m.add_class::<PyGeneration>()?;

    m.add_class::<PyAlterer>()?;
    m.add_class::<PySelector>()?;
    m.add_class::<PyDiversity>()?;
    m.add_class::<PyObjective>()?;
    m.add_class::<PyLimit>()?;
    m.add_class::<PySubscriber>()?;
    m.add_class::<PyEngineBuilder>()?;

    Ok(())
}
