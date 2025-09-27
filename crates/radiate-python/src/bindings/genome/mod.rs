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
pub use species::PySpecies;

use pyo3::{pyclass, pymethods};

pub const FLOAT_GENE_TYPE: &'static str = "FloatGene";
pub const INT_GENE_TYPE: &'static str = "IntGene";
pub const BIT_GENE_TYPE: &'static str = "BitGene";
pub const CHAR_GENE_TYPE: &'static str = "CharGene";
pub const GRAPH_GENE_TYPE: &'static str = "GraphNode";
pub const TREE_GENE_TYPE: &'static str = "TreeNode";
pub const PERMUTATION_GENE_TYPE: &'static str = "PermutationGene";
pub const ANY_GENE_TYPE: &'static str = "AnyGene";

#[pyclass]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum PyGeneType {
    Empty,
    Int,
    Float,
    Bit,
    Char,
    GraphNode,
    TreeNode,
    Permutation,
    AnyGene,
}

#[pymethods]
impl PyGeneType {
    pub fn name(&self) -> String {
        match self {
            PyGeneType::Empty => "NONE".into(),
            PyGeneType::Int => INT_GENE_TYPE.into(),
            PyGeneType::Float => FLOAT_GENE_TYPE.into(),
            PyGeneType::Bit => BIT_GENE_TYPE.into(),
            PyGeneType::Char => CHAR_GENE_TYPE.into(),
            PyGeneType::GraphNode => GRAPH_GENE_TYPE.into(),
            PyGeneType::TreeNode => TREE_GENE_TYPE.into(),
            PyGeneType::Permutation => PERMUTATION_GENE_TYPE.into(),
            PyGeneType::AnyGene => ANY_GENE_TYPE.into(),
        }
    }

    pub fn __repr__(&self) -> String {
        match self {
            PyGeneType::Empty => "NONE".into(),
            PyGeneType::Int => INT_GENE_TYPE.into(),
            PyGeneType::Float => FLOAT_GENE_TYPE.into(),
            PyGeneType::Bit => BIT_GENE_TYPE.into(),
            PyGeneType::Char => CHAR_GENE_TYPE.into(),
            PyGeneType::GraphNode => GRAPH_GENE_TYPE.into(),
            PyGeneType::TreeNode => TREE_GENE_TYPE.into(),
            PyGeneType::Permutation => PERMUTATION_GENE_TYPE.into(),
            PyGeneType::AnyGene => ANY_GENE_TYPE.into(),
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
            PyGeneType::Char => 4,
            PyGeneType::GraphNode => 5,
            PyGeneType::TreeNode => 6,
            PyGeneType::Permutation => 7,
            PyGeneType::AnyGene => 8,
        }
    }

    pub fn __eq__(&self, other: &PyGeneType) -> bool {
        self == other
    }

    pub fn __ne__(&self, other: &PyGeneType) -> bool {
        self != other
    }
}
