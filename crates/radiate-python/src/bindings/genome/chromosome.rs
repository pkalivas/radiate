use crate::{PyGene, PyGeneType};
use pyo3::{Bound, IntoPyObjectExt, PyAny, PyResult, Python, pyclass, pymethods};
use radiate::{
    BitChromosome, BitGene, CharChromosome, CharGene, Chromosome, FloatChromosome, FloatGene,
    GraphChromosome, GraphNode, IntChromosome, IntGene, Op, PermutationChromosome, PermutationGene,
    TreeChromosome, TreeNode,
};
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct GeneSequence {
    genes: Arc<RwLock<Vec<PyGene>>>,
}

impl GeneSequence {
    pub fn new(genes: Vec<PyGene>) -> Self {
        GeneSequence {
            genes: Arc::new(RwLock::new(genes)),
        }
    }

    pub fn read(&self) -> std::sync::RwLockReadGuard<'_, Vec<PyGene>> {
        self.genes.read().unwrap()
    }

    pub fn write(&self) -> std::sync::RwLockWriteGuard<'_, Vec<PyGene>> {
        self.genes.write().unwrap()
    }

    pub fn len(&self) -> usize {
        self.genes.read().unwrap().len()
    }

    pub fn drain(&self) -> Vec<PyGene> {
        self.genes.write().unwrap().drain(..).collect()
    }
}

impl PartialEq for GeneSequence {
    fn eq(&self, other: &Self) -> bool {
        *self.genes.read().unwrap() == *other.genes.read().unwrap()
    }
}

#[pyclass]
#[derive(Clone, Debug, PartialEq)]
#[repr(transparent)]
pub struct PyChromosome {
    pub(crate) genes: GeneSequence,
}

#[pymethods]
impl PyChromosome {
    #[new]
    pub fn new(genes: Vec<PyGene>) -> Self {
        PyChromosome {
            genes: GeneSequence::new(genes),
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "{:?}",
            self.genes
                .read()
                .iter()
                .map(|g| g.__repr__())
                .collect::<Vec<_>>()
        )
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn __len__(&self) -> usize {
        self.genes.len()
    }

    pub fn __eq__(&self, other: &Self) -> bool {
        self.genes
            .read()
            .iter()
            .zip(&*other.genes.read())
            .all(|(a, b)| a == b)
    }

    pub fn __getitem__<'py>(&self, py: Python<'py>, index: usize) -> PyResult<Bound<'py, PyAny>> {
        let reader = self.genes.read();
        let gene = reader
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

        if value.is_view() {
            self.genes.write()[index] = value.flatten();
        } else {
            self.genes.write()[index] = value;
        }

        Ok(())
    }

    pub fn view(&self, index: usize) -> PyResult<PyGene> {
        let cloned_genes = Arc::clone(&self.genes.genes);
        Ok(PyGene::from((cloned_genes, index)))
    }

    pub fn gene_type(&self) -> PyGeneType {
        let reader = self.genes.read();
        if reader.is_empty() {
            PyGeneType::Empty
        } else {
            reader[0].gene_type()
        }
    }
}

macro_rules! impl_into_py_chromosome {
    ($chromosome_type:ty, $gene_type:ty) => {
        impl From<$chromosome_type> for PyChromosome {
            fn from(chromosome: $chromosome_type) -> Self {
                PyChromosome {
                    genes: GeneSequence::new(
                        chromosome
                            .genes()
                            .iter()
                            .map(|gene| PyGene::from(gene.clone()))
                            .collect(),
                    ),
                }
            }
        }

        impl From<PyChromosome> for $chromosome_type {
            fn from(py_chromosome: PyChromosome) -> Self {
                let genes = if py_chromosome.genes.read().iter().any(|gene| gene.is_view()) {
                    py_chromosome
                        .genes
                        .read()
                        .iter()
                        .map(|gene| <$gene_type>::from(gene.clone()))
                        .collect::<Vec<_>>()
                } else {
                    py_chromosome
                        .genes
                        .drain()
                        .into_iter()
                        .map(|gene| <$gene_type>::from(gene))
                        .collect::<Vec<_>>()
                };
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
