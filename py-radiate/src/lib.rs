use pyo3::prelude::*;
use radiate_python::{
    PyAnyCodec, PyBitCodec, PyCharCodec, PyChromosome, PyEngine, PyEngineBuilder, PyEngineInput,
    PyEngineInputType, PyFloatCodec, PyGene, PyGeneType, PyGeneration, PyGenotype, PyGraph,
    PyGraphCodec, PyIntCodec, PyPermutationCodec, PyPhenotype, PyPopulation, PyProblemBuilder,
    PyRandomProvider, PySubscriber, PyTree, PyTreeCodec, py_alter, py_select,
};

#[pymodule]
fn radiate(m: &Bound<'_, PyModule>) -> PyResult<()> {
    radiate_python::init_logging();

    m.add_function(wrap_pyfunction!(py_select, m)?)?;
    m.add_function(wrap_pyfunction!(py_alter, m)?)?;

    m.add_class::<PyRandomProvider>()?;
    m.add_class::<PyProblemBuilder>()?;

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
    m.add_class::<PyGraphCodec>()?;
    m.add_class::<PyTreeCodec>()?;
    m.add_class::<PyPermutationCodec>()?;
    m.add_class::<PyAnyCodec>()?;

    m.add_class::<PySubscriber>()?;

    m.add_class::<PyGraph>()?;
    m.add_class::<PyTree>()?;

    m.add_class::<PyEngineInputType>()?;
    m.add_class::<PyEngineInput>()?;
    m.add_class::<PyEngineBuilder>()?;
    m.add_class::<PyEngine>()?;
    m.add_class::<PyGeneration>()?;

    Ok(())
}
