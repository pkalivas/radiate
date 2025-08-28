use std::sync::{Arc, RwLock};

use crate::{PyGene, PyGeneType};
use pyo3::{Bound, IntoPyObjectExt, PyAny, PyResult, Python, pyclass, pymethods};
use radiate::{
    BitChromosome, BitGene, CharChromosome, CharGene, Chromosome, FloatChromosome, FloatGene,
    GraphChromosome, GraphNode, IntChromosome, IntGene, Op, PermutationChromosome, PermutationGene,
    TreeChromosome, TreeNode,
};

pub struct PyGeneView {
    genes: Arc<RwLock<Vec<PyGene>>>,
}

impl PartialEq for PyGeneView {
    fn eq(&self, other: &Self) -> bool {
        (*self.genes.read().unwrap()) == (*other.genes.read().unwrap())
    }
}

#[pyclass]
#[derive(Clone, Debug, PartialEq)]
#[repr(transparent)]
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

    pub fn __getitem__<'py>(&self, py: Python<'py>, index: usize) -> PyResult<Bound<'py, PyAny>> {
        let gene = self
            .genes
            .get(index)
            .ok_or_else(|| pyo3::exceptions::PyIndexError::new_err("index out of range"))?;

        gene.clone().into_bound_py_any(py)
    }

    pub fn __setitem__(&mut self, index: usize, value: PyGene) -> PyResult<()> {
        if index >= self.genes.len() {
            return Err(pyo3::exceptions::PyIndexError::new_err(
                "index out of range",
            ));
        }

        if self.gene_type() != value.gene_type() {
            return Err(pyo3::exceptions::PyTypeError::new_err(format!(
                "expected gene of type {:?}, got {:?}",
                self.gene_type(),
                value.gene_type()
            )));
        }

        self.genes[index] = value;
        Ok(())
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
                        .genes()
                        .iter()
                        .map(|gene| PyGene::from(gene.clone()))
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
