mod chromosomes;
mod genotype;

pub use chromosomes::{PyFloatChromosome, PyFloatGene, PyIntChromosome, PyIntGene};
use genotype::PyGenotype;
use pyo3::pyclass;
use radiate::Chromosome;
