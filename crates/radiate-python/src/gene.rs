use pyo3::{pyclass, pymethods};

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
    pub fn __repr__(&self) -> String {
        match self {
            PyGeneType::Int => "GeneType.Int".into(),
            PyGeneType::Float => "GeneType.Float".into(),
            PyGeneType::Bit => "GeneType.Bit".into(),
            PyGeneType::Char => "GeneType.Char".into(),
            PyGeneType::Any => "GeneType.Any".into(),
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
