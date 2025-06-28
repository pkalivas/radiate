use pyo3::pyclass;
use radiate::{FloatChromosome, FloatGene};

#[pyclass]
#[derive(Clone, Debug, PartialEq)]
pub struct PyFloatGene {
    inner: FloatGene,
}

#[pyclass]
#[derive(Clone, Debug, PartialEq)]
pub struct PyFloatChromosome {
    inner: FloatChromosome,
}
