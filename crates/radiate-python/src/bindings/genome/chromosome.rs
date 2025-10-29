use crate::{AnyChromosome, AnyGene, PyGene, PyGeneType};
use pyo3::{PyResult, exceptions::PyIndexError, pyclass, pymethods};
use radiate::prelude::*;

#[pyclass]
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

    pub fn __getitem__<'py>(&self, index: isize) -> PyResult<PyGene> {
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

impl_into_py_chromosome!(FloatChromosome, FloatGene);
impl_into_py_chromosome!(IntChromosome<i32>, IntGene<i32>);
impl_into_py_chromosome!(BitChromosome, BitGene);
impl_into_py_chromosome!(CharChromosome, CharGene);
impl_into_py_chromosome!(GraphChromosome<Op<f32>>, GraphNode<Op<f32>>);
impl_into_py_chromosome!(TreeChromosome<Op<f32>>, TreeNode<Op<f32>>);
impl_into_py_chromosome!(PermutationChromosome<usize>, PermutationGene<usize>);
impl_into_py_chromosome!(AnyChromosome<'static>, AnyGene<'static>);
