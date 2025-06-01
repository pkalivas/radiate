use pyo3::prelude::*;
use radiate_python::{
    PyAlterer, PyBitCodec, PyCharCodec, PyDiversity, PyEngineBuilder, PyFloatCodec, PyGene,
    PyGeneType, PyGeneration, PyIntCodec, PyLimit, PyObjective, PyRandomProvider, PySelector,
    PySubscriber,
};

#[pymodule]
fn radiate(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyRandomProvider>()?;

    m.add_class::<PyGeneType>()?;
    m.add_class::<PyGene>()?;

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
