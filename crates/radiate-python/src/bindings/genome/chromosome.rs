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

// use crate::{AnyChromosome, AnyGene, PyGene, PyGeneType, RwSequence};
// use pyo3::{
//     Bound, IntoPyObjectExt, Py, PyAny, PyRef, PyResult, Python,
//     exceptions::{PyIndexError, PyTypeError, PyValueError},
//     pyclass, pymethods,
//     types::{PyAnyMethods, PySlice, PySliceMethods},
// };
// use radiate::prelude::*;

// #[pyclass]
// pub struct PyGeneIterator {
//     current: usize,
//     genes: RwSequence<PyGene>,
// }

// #[pymethods]
// impl PyGeneIterator {
//     fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
//         slf
//     }

//     fn __next__(&mut self) -> Option<PyGene> {
//         if self.current < self.genes.len() {
//             let gene = PyGene::from((self.genes.clone(), self.current));
//             self.current += 1;
//             Some(gene)
//         } else {
//             None
//         }
//     }
// }

// #[pyclass]
// #[derive(Clone, Debug, PartialEq)]
// pub struct PyChromosome {
//     pub(crate) genes: RwSequence<PyGene>,
// }

// #[pymethods]
// impl PyChromosome {
//     #[new]
//     pub fn new(genes: Vec<PyGene>) -> Self {
//         PyChromosome {
//             genes: RwSequence::new(genes),
//         }
//     }

//     pub fn __repr__(&self) -> String {
//         format!(
//             "{:?}",
//             self.genes
//                 .read()
//                 .iter()
//                 .map(|g| g.__repr__())
//                 .collect::<Vec<_>>()
//         )
//     }

//     pub fn __str__(&self) -> String {
//         self.__repr__()
//     }

//     pub fn __len__(&self) -> usize {
//         self.genes.len()
//     }

//     pub fn __eq__(&self, other: &Self) -> bool {
//         self.genes
//             .read()
//             .iter()
//             .zip(&*other.genes.read())
//             .all(|(a, b)| a == b)
//     }

//     pub fn __iter__(&self) -> PyGeneIterator {
//         PyGeneIterator {
//             current: 0,
//             genes: self.genes.clone(),
//         }
//     }

//     pub fn __getitem__<'py>(
//         &self,
//         py: Python<'py>,
//         index: Py<PyAny>,
//     ) -> PyResult<Bound<'py, PyAny>> {
//         let index = index.into_bound(py);
//         if index.is_instance_of::<PySlice>() {
//             return self.get_item(py, index, true);
//         }

//         self.get_item(py, index, false)
//     }

//     pub fn __setitem__<'py>(
//         &mut self,
//         py: Python<'py>,
//         index: Py<PyAny>,
//         value: Py<PyAny>,
//     ) -> PyResult<()> {
//         self.set_item(py, index, value)
//     }

//     #[pyo3(signature = (index=None))]
//     pub fn copy<'py>(
//         &self,
//         py: Python<'py>,
//         index: Option<Py<PyAny>>,
//     ) -> PyResult<Bound<'py, PyAny>> {
//         match index {
//             Some(idx) => self.get_item(py, idx.into_bound(py), false),
//             None => PyChromosome {
//                 genes: RwSequence::new(self.genes.read().clone()),
//             }
//             .into_bound_py_any(py),
//         }
//     }

//     #[pyo3(signature = (index=None))]
//     pub fn view<'py>(
//         &self,
//         py: Python<'py>,
//         index: Option<Py<PyAny>>,
//     ) -> PyResult<Bound<'py, PyAny>> {
//         match index {
//             Some(idx) => self.get_item(py, idx.into_bound(py), true),
//             None => PyChromosome {
//                 genes: self.genes.clone(),
//             }
//             .into_bound_py_any(py),
//         }
//     }

//     fn apply<'py>(&self, py: Python<'py>, f: Py<PyAny>) -> PyResult<()> {
//         for i in 0..self.genes.len() {
//             self.genes.write()[i].apply(py, f.clone_ref(py))?;
//         }

//         Ok(())
//     }

//     pub fn map<'py>(&self, py: Python<'py>, f: Py<PyAny>) -> PyResult<PyChromosome> {
//         for i in 0..self.genes.len() {
//             let mut writer = self.genes.write();
//             let new_gene = writer[i].map(py, f.clone_ref(py))?;
//             writer[i] = new_gene;
//         }

//         Ok(self.clone())
//     }

//     pub fn is_view(&self) -> bool {
//         let ref_count = self.genes.strong_count() + self.genes.weak_count();
//         self.genes.read().iter().any(|gene| gene.is_view()) || ref_count > 1
//     }

//     pub fn gene_type(&self) -> PyGeneType {
//         let reader = self.genes.read();
//         if reader.is_empty() {
//             PyGeneType::Empty
//         } else {
//             reader[0].gene_type()
//         }
//     }

//     fn get_item<'py>(
//         &self,
//         py: Python<'py>,
//         index: Bound<'py, PyAny>,
//         view: bool,
//     ) -> PyResult<Bound<'py, PyAny>> {
//         if let Ok(mut idx) = index.extract::<isize>() {
//             let n = self.genes.len() as isize;
//             if idx < 0 {
//                 idx += n;
//             }

//             if idx < 0 || idx >= n {
//                 return Err(PyIndexError::new_err("index out of range"));
//             }

//             if view {
//                 return PyGene::from((self.genes.clone(), idx as usize)).into_bound_py_any(py);
//             } else {
//                 return self.genes.read()[idx as usize]
//                     .clone()
//                     .into_bound_py_any(py);
//             }
//         }

//         if let Ok(py_slice) = index.downcast::<PySlice>() {
//             let indices = py_slice
//                 .indices(self.genes.len() as isize)
//                 .map_err(|e| PyIndexError::new_err(format!("invalid slice: {}", e)))?;

//             let (start, stop, step) = (
//                 indices.start as usize,
//                 indices.stop as usize,
//                 indices.step as usize,
//             );

//             let mut result = Vec::with_capacity(((stop.saturating_sub(start)) + step - 1) / step);
//             for i in (start..stop).step_by(step) {
//                 result.push(if view {
//                     PyGene::from((self.genes.clone(), i))
//                 } else {
//                     self.genes.read()[i].clone()
//                 });
//             }

//             return PyChromosome {
//                 genes: RwSequence::new(result),
//             }
//             .into_bound_py_any(py);
//         }

//         Err(PyTypeError::new_err("invalid index type"))
//     }

//     fn set_item<'py>(
//         &mut self,
//         py: Python<'py>,
//         index: Py<PyAny>,
//         value: Py<PyAny>,
//     ) -> PyResult<()> {
//         if let Ok(mut idx) = index.extract::<isize>(py) {
//             let n = self.genes.len() as isize;

//             if idx < 0 {
//                 idx += n;
//             }

//             if idx < 0 || idx >= n {
//                 return Err(PyIndexError::new_err("index out of range"));
//             }

//             let mut v = value.extract::<PyGene>(py)?;
//             if self.gene_type() != v.gene_type() {
//                 return Err(PyTypeError::new_err("gene type mismatch"));
//             }

//             if v.is_view() {
//                 v = v.flatten();
//             }

//             self.genes.write()[idx as usize] = v;

//             return Ok(());
//         }

//         let bound = index.into_bound(py);
//         if let Ok(py_slice) = bound.downcast::<PySlice>() {
//             let indices = py_slice
//                 .indices(self.genes.len() as isize)
//                 .map_err(|e| PyIndexError::new_err(format!("invalid slice: {}", e)))?;
//             let (start, stop, step) = (
//                 indices.start as usize,
//                 indices.stop as usize,
//                 indices.step as usize,
//             );

//             let value = value.extract::<PyChromosome>(py)?;
//             let expected = ((stop.saturating_sub(start)) + step - 1) / step;

//             if value.genes.len() != expected {
//                 return Err(PyValueError::new_err("slice assignment length mismatch"));
//             }
//             if self.gene_type() != value.gene_type() {
//                 return Err(PyTypeError::new_err("gene type mismatch"));
//             }
//             let vals = value.genes.read();
//             let mut w = self.genes.write();
//             let mut vi = 0;
//             for i in (start..stop).step_by(step) {
//                 let mut v = vals[vi].clone();
//                 if v.is_view() {
//                     v = v.flatten();
//                 }
//                 w[i] = v;
//                 vi += 1;
//             }
//             return Ok(());
//         }

//         Err(PyTypeError::new_err("invalid index type"))
//     }
// }

// macro_rules! impl_into_py_chromosome {
//     ($chromosome_type:ty, $gene_type:ty) => {
//         impl From<$chromosome_type> for PyChromosome {
//             fn from(chromosome: $chromosome_type) -> Self {
//                 PyChromosome {
//                     genes: RwSequence::new(
//                         chromosome
//                             .into_iter()
//                             .map(|gene| PyGene::from(gene))
//                             .collect(),
//                     ),
//                 }
//             }
//         }

//         impl From<PyChromosome> for $chromosome_type {
//             fn from(py_chromosome: PyChromosome) -> Self {
//                 if py_chromosome.is_view() {
//                     <$chromosome_type>::from(
//                         py_chromosome
//                             .genes
//                             .read()
//                             .iter()
//                             .cloned()
//                             .map(|gene| {
//                                 if gene.is_view() {
//                                     <$gene_type>::from(gene.flatten())
//                                 } else {
//                                     <$gene_type>::from(gene)
//                                 }
//                             })
//                             .collect::<Vec<$gene_type>>(),
//                     )
//                 } else {
//                     let genes = py_chromosome
//                         .genes
//                         .take()
//                         .into_iter()
//                         .map(|gene| <$gene_type>::from(gene))
//                         .collect::<Vec<_>>();
//                     <$chromosome_type>::from(genes)
//                 }
//             }
//         }
//     };
// }

// impl_into_py_chromosome!(FloatChromosome, FloatGene);
// impl_into_py_chromosome!(IntChromosome<i32>, IntGene<i32>);
// impl_into_py_chromosome!(BitChromosome, BitGene);
// impl_into_py_chromosome!(CharChromosome, CharGene);
// impl_into_py_chromosome!(GraphChromosome<Op<f32>>, GraphNode<Op<f32>>);
// impl_into_py_chromosome!(TreeChromosome<Op<f32>>, TreeNode<Op<f32>>);
// impl_into_py_chromosome!(PermutationChromosome<usize>, PermutationGene<usize>);
// impl_into_py_chromosome!(AnyChromosome<'static>, AnyGene<'static>);
