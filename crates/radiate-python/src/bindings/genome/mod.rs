mod chromosome;
mod ecosystem;
mod gene;
mod genotype;
mod phenotype;
mod population;
mod species;

pub use chromosome::PyChromosome;
pub use ecosystem::PyEcosystem;
pub use gene::PyGene;
pub use genotype::PyGenotype;
pub use phenotype::PyPhenotype;
pub use population::PyPopulation;
use serde::{Deserialize, Serialize};
pub use species::PySpecies;

use pyo3::{pyclass, pymethods};

pub const FLOAT_GENE_TYPE: &str = "FloatGene";
pub const INT_GENE_TYPE: &str = "IntGene";
pub const BIT_GENE_TYPE: &str = "BitGene";
pub const PACKED_BIT_GENE_TYPE: &str = "PackedBitGene";
pub const CHAR_GENE_TYPE: &str = "CharGene";
pub const GRAPH_GENE_TYPE: &str = "GraphNode";
pub const TREE_GENE_TYPE: &str = "TreeNode";
pub const PERMUTATION_GENE_TYPE: &str = "PermutationGene";

#[pyclass(from_py_object)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy, Serialize, Deserialize)]
pub enum PyGeneType {
    Empty,
    Int,
    Float,
    Bit,
    PackedBit,
    Char,
    GraphNode,
    TreeNode,
    Permutation,
}

#[pymethods]
impl PyGeneType {
    pub fn name(&self) -> String {
        match self {
            PyGeneType::Empty => "NONE".into(),
            PyGeneType::Int => INT_GENE_TYPE.into(),
            PyGeneType::Float => FLOAT_GENE_TYPE.into(),
            PyGeneType::Bit => BIT_GENE_TYPE.into(),
            PyGeneType::PackedBit => PACKED_BIT_GENE_TYPE.into(),
            PyGeneType::Char => CHAR_GENE_TYPE.into(),
            PyGeneType::GraphNode => GRAPH_GENE_TYPE.into(),
            PyGeneType::TreeNode => TREE_GENE_TYPE.into(),
            PyGeneType::Permutation => PERMUTATION_GENE_TYPE.into(),
        }
    }

    pub fn __repr__(&self) -> String {
        match self {
            PyGeneType::Empty => "NONE".into(),
            PyGeneType::Int => INT_GENE_TYPE.into(),
            PyGeneType::Float => FLOAT_GENE_TYPE.into(),
            PyGeneType::Bit => BIT_GENE_TYPE.into(),
            PyGeneType::PackedBit => PACKED_BIT_GENE_TYPE.into(),
            PyGeneType::Char => CHAR_GENE_TYPE.into(),
            PyGeneType::GraphNode => GRAPH_GENE_TYPE.into(),
            PyGeneType::TreeNode => TREE_GENE_TYPE.into(),
            PyGeneType::Permutation => PERMUTATION_GENE_TYPE.into(),
        }
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn __hash__(&self) -> usize {
        match self {
            PyGeneType::Empty => 0,
            PyGeneType::Int => 1,
            PyGeneType::Float => 2,
            PyGeneType::Bit => 3,
            PyGeneType::PackedBit => 4,
            PyGeneType::Char => 5,
            PyGeneType::GraphNode => 6,
            PyGeneType::TreeNode => 7,
            PyGeneType::Permutation => 8,
        }
    }

    pub fn __eq__(&self, other: &PyGeneType) -> bool {
        self == other
    }

    pub fn __ne__(&self, other: &PyGeneType) -> bool {
        self != other
    }
}
