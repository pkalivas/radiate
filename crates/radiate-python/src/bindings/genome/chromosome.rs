use crate::{PyGene, PyGeneType, Wrap};
use pyo3::{
    Bound, IntoPyObject, PyAny, PyResult, Python, exceptions::PyIndexError, pyclass, pymethods,
};
use radiate::prelude::*;
use radiate_utils::DataType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ChromosomeInner {
    UInt8(IntChromosome<u8>),
    UInt16(IntChromosome<u16>),
    UInt32(IntChromosome<u32>),
    UInt64(IntChromosome<u64>),
    UInt128(IntChromosome<u128>),
    Int8(IntChromosome<i8>),
    Int16(IntChromosome<i16>),
    Int32(IntChromosome<i32>),
    Int64(IntChromosome<i64>),
    Int128(IntChromosome<i128>),

    Float32(FloatChromosome<f32>),
    Float64(FloatChromosome<f64>),

    Bit(BitChromosome),
    Char(CharChromosome),

    Permutation(PermutationChromosome<usize>),

    GraphNode(GraphChromosome<Op<f32>>),
    TreeNode(TreeChromosome<Op<f32>>),
}

#[pyclass(from_py_object)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyChromosome {
    pub(crate) genes: Vec<PyGene>,
}

#[pymethods]
impl PyChromosome {
    #[new]
    pub fn new(genes: Vec<PyGene>) -> Self {
        PyChromosome { genes }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "{:?}",
            self.genes.iter().map(|g| g.__repr__()).collect::<Vec<_>>()
        )
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn __len__(&self) -> usize {
        self.genes.len()
    }

    pub fn __eq__(&self, other: &Self) -> bool {
        self.genes.iter().zip(&other.genes).all(|(a, b)| a == b)
    }

    pub fn __getitem__(&self, index: isize) -> PyResult<PyGene> {
        if index >= self.genes.len() as isize || index < -(self.genes.len() as isize) {
            return Err(PyIndexError::new_err("index out of range"));
        }

        let index = if index < 0 {
            index + self.genes.len() as isize
        } else {
            index
        } as usize;

        Ok(self.genes[index].clone())
    }

    pub fn gene_type(&self) -> PyGeneType {
        if self.genes.is_empty() {
            PyGeneType::Empty
        } else {
            self.genes[0].gene_type()
        }
    }

    pub fn dtype<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        if self.genes.is_empty() {
            Wrap(DataType::Null).into_pyobject(py)
        } else {
            self.genes[0].dtype(py)
        }
    }
}

macro_rules! impl_into_py_chromosome {
    ($chromosome_type:ty, $gene_type:ty) => {
        impl From<$chromosome_type> for PyChromosome {
            fn from(chromosome: $chromosome_type) -> Self {
                PyChromosome {
                    genes: chromosome
                        .into_iter()
                        .map(|gene| PyGene::from(gene))
                        .collect(),
                }
            }
        }

        impl From<PyChromosome> for $chromosome_type {
            fn from(py_chromosome: PyChromosome) -> Self {
                let genes = py_chromosome
                    .genes
                    .into_iter()
                    .map(|gene| <$gene_type>::from(gene))
                    .collect::<Vec<_>>();
                <$chromosome_type>::from(genes)
            }
        }
    };
}

impl_into_py_chromosome!(IntChromosome<u8>, IntGene<u8>);
impl_into_py_chromosome!(IntChromosome<u16>, IntGene<u16>);
impl_into_py_chromosome!(IntChromosome<u32>, IntGene<u32>);
impl_into_py_chromosome!(IntChromosome<u64>, IntGene<u64>);
impl_into_py_chromosome!(IntChromosome<u128>, IntGene<u128>);

impl_into_py_chromosome!(IntChromosome<i8>, IntGene<i8>);
impl_into_py_chromosome!(IntChromosome<i16>, IntGene<i16>);
impl_into_py_chromosome!(IntChromosome<i32>, IntGene<i32>);
impl_into_py_chromosome!(IntChromosome<i64>, IntGene<i64>);
impl_into_py_chromosome!(IntChromosome<i128>, IntGene<i128>);

impl_into_py_chromosome!(FloatChromosome<f32>, FloatGene<f32>);
impl_into_py_chromosome!(FloatChromosome<f64>, FloatGene<f64>);

impl_into_py_chromosome!(BitChromosome, BitGene);
impl_into_py_chromosome!(CharChromosome, CharGene);
impl_into_py_chromosome!(GraphChromosome<Op<f32>>, GraphNode<Op<f32>>);
impl_into_py_chromosome!(TreeChromosome<Op<f32>>, TreeNode<Op<f32>>);
impl_into_py_chromosome!(PermutationChromosome<usize>, PermutationGene<usize>);
