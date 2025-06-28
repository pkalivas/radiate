mod float;
mod gene;

pub use gene::{PyChromosome, PyGene, PyGenotype, PyPhenotype, PyPopulation};

use pyo3::{pyclass, pymethods};

pub const FLOAT_GENE_TYPE: &'static str = "FloatGene";
pub const INT_GENE_TYPE: &'static str = "IntGene";
pub const BIT_GENE_TYPE: &'static str = "BitGene";
pub const CHAR_GENE_TYPE: &'static str = "CharGene";

pub const FLOAT_CHROMOSOME_TYPE: &'static str = "FloatChromosome";
pub const INT_CHROMOSOME_TYPE: &'static str = "IntChromosome";
pub const BIT_CHROMOSOME_TYPE: &'static str = "BitChromosome";
pub const CHAR_CHROMOSOME_TYPE: &'static str = "CharChromosome";

#[pyclass]
#[derive(Clone, Debug, PartialEq)]
pub enum PyChromosomeType {
    Float,
    Int,
    Bit,
    Char,
}

#[pymethods]
impl PyChromosomeType {
    pub fn name(&self) -> String {
        match self {
            PyChromosomeType::Float => FLOAT_CHROMOSOME_TYPE.into(),
            PyChromosomeType::Int => INT_CHROMOSOME_TYPE.into(),
            PyChromosomeType::Bit => BIT_CHROMOSOME_TYPE.into(),
            PyChromosomeType::Char => CHAR_CHROMOSOME_TYPE.into(),
        }
    }

    pub fn __repr__(&self) -> String {
        match self {
            PyChromosomeType::Float => FLOAT_CHROMOSOME_TYPE.into(),
            PyChromosomeType::Int => INT_CHROMOSOME_TYPE.into(),
            PyChromosomeType::Bit => BIT_CHROMOSOME_TYPE.into(),
            PyChromosomeType::Char => CHAR_CHROMOSOME_TYPE.into(),
        }
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn __hash__(&self) -> usize {
        match self {
            PyChromosomeType::Float => 0,
            PyChromosomeType::Int => 1,
            PyChromosomeType::Bit => 2,
            PyChromosomeType::Char => 3,
        }
    }

    pub fn __eq__(&self, other: &PyChromosomeType) -> bool {
        self == other
    }

    pub fn __ne__(&self, other: &PyChromosomeType) -> bool {
        self != other
    }
}

#[pyclass]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum PyGeneType {
    Int,
    Float,
    Bit,
    Char,
}

#[pymethods]
impl PyGeneType {
    pub fn name(&self) -> String {
        match self {
            PyGeneType::Int => INT_GENE_TYPE.into(),
            PyGeneType::Float => FLOAT_GENE_TYPE.into(),
            PyGeneType::Bit => BIT_GENE_TYPE.into(),
            PyGeneType::Char => CHAR_GENE_TYPE.into(),
        }
    }

    pub fn __repr__(&self) -> String {
        match self {
            PyGeneType::Int => INT_GENE_TYPE.into(),
            PyGeneType::Float => FLOAT_GENE_TYPE.into(),
            PyGeneType::Bit => BIT_GENE_TYPE.into(),
            PyGeneType::Char => CHAR_GENE_TYPE.into(),
        }
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn __hash__(&self) -> usize {
        match self {
            PyGeneType::Int => 0,
            PyGeneType::Float => 1,
            PyGeneType::Bit => 2,
            PyGeneType::Char => 3,
        }
    }

    pub fn __eq__(&self, other: &PyGeneType) -> bool {
        self == other
    }

    pub fn __ne__(&self, other: &PyGeneType) -> bool {
        self != other
    }
}
