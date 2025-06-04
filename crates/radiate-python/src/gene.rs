use pyo3::{pyclass, pymethods};

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
            PyChromosomeType::Float => "FloatChromosome".into(),
            PyChromosomeType::Int => "IntChromosome".into(),
            PyChromosomeType::Bit => "BitChromosome".into(),
            PyChromosomeType::Char => "CharChromosome".into(),
        }
    }

    pub fn __repr__(&self) -> String {
        match self {
            PyChromosomeType::Float => "FloatChromosome".into(),
            PyChromosomeType::Int => "IntChromosome".into(),
            PyChromosomeType::Bit => "BitChromosome".into(),
            PyChromosomeType::Char => "CharChromosome".into(),
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
    Any,
}

#[pymethods]
impl PyGeneType {
    pub fn name(&self) -> String {
        match self {
            PyGeneType::Int => "IntGene".into(),
            PyGeneType::Float => "FloatGene".into(),
            PyGeneType::Bit => "BitGene".into(),
            PyGeneType::Char => "CharGene".into(),
            PyGeneType::Any => "AnyGene".into(),
        }
    }

    pub fn __repr__(&self) -> String {
        match self {
            PyGeneType::Int => "IntGene".into(),
            PyGeneType::Float => "FloatGene".into(),
            PyGeneType::Bit => "BitGene".into(),
            PyGeneType::Char => "CharGene".into(),
            PyGeneType::Any => "AnyGene".into(),
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
            PyGeneType::Any => 4,
        }
    }

    pub fn __eq__(&self, other: &PyGeneType) -> bool {
        self == other
    }

    pub fn __ne__(&self, other: &PyGeneType) -> bool {
        self != other
    }
}
