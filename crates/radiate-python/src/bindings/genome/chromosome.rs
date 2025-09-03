use crate::{AnyChromosome, AnyGene, PyGene, PyGeneType};
use pyo3::{
    Bound, IntoPyObjectExt, Py, PyAny, PyResult, Python,
    exceptions::{PyIndexError, PyTypeError},
    pyclass, pymethods,
    types::{PyAnyMethods, PySlice, PySliceMethods},
};
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

    pub fn __getitem__<'py>(
        &self,
        py: Python<'py>,
        index: Py<PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let index = index.into_bound(py);
        self.get_item(py, index)
    }

    pub fn gene_type(&self) -> PyGeneType {
        if self.genes.is_empty() {
            PyGeneType::Empty
        } else {
            self.genes[0].gene_type()
        }
    }

    fn get_item<'py>(
        &self,
        py: Python<'py>,
        index: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if let Ok(mut idx) = index.extract::<isize>() {
            let n = self.genes.len() as isize;
            if idx < 0 {
                idx += n;
            }

            if idx < 0 || idx >= n {
                return Err(PyIndexError::new_err("index out of range"));
            }

            return self.genes[idx as usize].clone().into_bound_py_any(py);
        }

        if let Ok(py_slice) = index.downcast::<PySlice>() {
            let indices = py_slice
                .indices(self.genes.len() as isize)
                .map_err(|e| PyIndexError::new_err(format!("invalid slice: {}", e)))?;

            let (start, stop, step) = (
                indices.start as usize,
                indices.stop as usize,
                indices.step as usize,
            );

            let mut result = Vec::with_capacity(((stop.saturating_sub(start)) + step - 1) / step);
            for i in (start..stop).step_by(step) {
                result.push(self.genes[i].clone());
            }

            return PyChromosome { genes: result }.into_bound_py_any(py);
        }

        Err(PyTypeError::new_err("invalid index type"))
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
