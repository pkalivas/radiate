use super::{PyFloatChromosome, PyIntChromosome};
use pyo3::pyclass;

#[pyclass]
pub enum PyGenotype {
    Float(Vec<PyFloatChromosome>),
    Int(Vec<PyIntChromosome>),
}
