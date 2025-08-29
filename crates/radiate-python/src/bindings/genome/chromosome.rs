use crate::{AnyChromosome, AnyGene, PyGene, PyGeneType};
use pyo3::{
    Bound, IntoPyObjectExt, Py, PyAny, PyResult, Python, pyclass, pymethods,
    types::{PyAnyMethods, PySlice, PySliceMethods},
};
use radiate::{
    BitChromosome, BitGene, CharChromosome, CharGene, FloatChromosome, FloatGene, GraphChromosome,
    GraphNode, IntChromosome, IntGene, Op, PermutationChromosome, PermutationGene, TreeChromosome,
    TreeNode,
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
}

impl PartialEq for GeneSequence {
    fn eq(&self, other: &Self) -> bool {
        *self.genes.read().unwrap() == *other.genes.read().unwrap()
    }
}

#[pyclass]
#[derive(Clone, Debug, PartialEq)]
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

    pub fn __getitem__<'py>(
        &self,
        py: Python<'py>,
        index: Py<PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        // accept negative indices
        if let Ok(mut idx) = index.extract::<isize>(py) {
            let n = self.genes.len() as isize;
            if idx < 0 {
                idx += n;
            }
            if idx < 0 || idx >= n {
                return Err(pyo3::exceptions::PyIndexError::new_err(
                    "index out of range",
                ));
            }
            return PyGene::from((Arc::clone(&self.genes.genes), idx as usize))
                .into_bound_py_any(py);
        }
        let bound = index.into_bound(py);
        if let Ok(py_slice) = bound.downcast::<PySlice>() {
            let indices = py_slice.indices(self.genes.len() as isize).map_err(|e| {
                pyo3::exceptions::PyIndexError::new_err(format!("invalid slice: {}", e))
            })?;
            let (start, stop, step) = (
                indices.start as usize,
                indices.stop as usize,
                indices.step as usize,
            );
            let mut result = Vec::with_capacity(((stop.saturating_sub(start)) + step - 1) / step);
            for i in (start..stop).step_by(step) {
                result.push(PyGene::from((Arc::clone(&self.genes.genes), i)));
            }
            return PyChromosome {
                genes: GeneSequence::new(result),
            }
            .into_bound_py_any(py);
        }
        Err(pyo3::exceptions::PyTypeError::new_err("invalid index type"))
    }

    pub fn __setitem__<'py>(
        &mut self,
        py: Python<'py>,
        index: Py<PyAny>,
        value: Py<PyAny>,
    ) -> PyResult<()> {
        if let Ok(mut idx) = index.extract::<isize>(py) {
            let n = self.genes.len() as isize;
            if idx < 0 {
                idx += n;
            }
            if idx < 0 || idx >= n {
                return Err(pyo3::exceptions::PyIndexError::new_err(
                    "index out of range",
                ));
            }
            let mut v = value.extract::<PyGene>(py)?;
            if self.gene_type() != v.gene_type() {
                return Err(pyo3::exceptions::PyTypeError::new_err("gene type mismatch"));
            }
            if v.is_view() {
                v = v.flatten();
            }
            self.genes.write()[idx as usize] = v;
            return Ok(());
        }

        let bound = index.into_bound(py);
        if let Ok(py_slice) = bound.downcast::<PySlice>() {
            let indices = py_slice.indices(self.genes.len() as isize).map_err(|e| {
                pyo3::exceptions::PyIndexError::new_err(format!("invalid slice: {}", e))
            })?;
            let (start, stop, step) = (
                indices.start as usize,
                indices.stop as usize,
                indices.step as usize,
            );
            let value = value.extract::<PyChromosome>(py)?;
            // length check
            let expected = ((stop.saturating_sub(start)) + step - 1) / step;
            if value.genes.len() != expected {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "slice assignment length mismatch",
                ));
            }
            if self.gene_type() != value.gene_type() {
                return Err(pyo3::exceptions::PyTypeError::new_err("gene type mismatch"));
            }
            let vals = value.genes.read(); // read RHS once
            let mut w = self.genes.write(); // write LHS once
            let mut vi = 0usize;
            for i in (start..stop).step_by(step) {
                let mut v = vals[vi].clone();
                if v.is_view() {
                    v = v.flatten();
                }
                w[i] = v;
                vi += 1;
            }
            return Ok(());
        }
        Err(pyo3::exceptions::PyTypeError::new_err("invalid index type"))
    }

    pub fn gene_type(&self) -> PyGeneType {
        let reader = self.genes.read();
        if reader.is_empty() {
            PyGeneType::Empty
        } else {
            reader[0].gene_type()
        }
    }

    pub fn allele<'py>(&self, py: Python<'py>, index: usize) -> PyResult<Bound<'py, PyAny>> {
        let reader = self.genes.read();
        if index >= reader.len() {
            return Err(pyo3::exceptions::PyIndexError::new_err(
                "index out of range",
            ));
        }

        reader[index].allele(py)
    }

    pub fn set_allele<'py>(
        &mut self,
        py: Python<'py>,
        index: usize,
        value: Option<Py<PyAny>>,
    ) -> PyResult<()> {
        let mut writer = self.genes.write();
        if index >= writer.len() {
            return Err(pyo3::exceptions::PyIndexError::new_err(
                "index out of range",
            ));
        }

        writer[index] = writer[index].new_instance(py, value)?;
        Ok(())
    }
}

macro_rules! impl_into_py_chromosome {
    ($chromosome_type:ty, $gene_type:ty) => {
        impl From<$chromosome_type> for PyChromosome {
            fn from(chromosome: $chromosome_type) -> Self {
                PyChromosome {
                    genes: GeneSequence::new(
                        chromosome
                            .into_iter()
                            .map(|gene| PyGene::from(gene))
                            .collect(),
                    ),
                }
            }
        }

        impl From<PyChromosome> for $chromosome_type {
            fn from(py_chromosome: PyChromosome) -> Self {
                let genes = py_chromosome
                    .genes
                    .read()
                    .iter()
                    .map(|gene| {
                        if gene.is_view() {
                            <$gene_type>::from(gene.flatten())
                        } else {
                            <$gene_type>::from(gene.clone())
                        }
                    })
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
