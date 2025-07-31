use std::clone;

use crate::{
    ObjectValue, PyEngineInput, PyGeneType, PyNoveltySearch,
    bindings::fitness::novelty::PyDescriptor,
};
use pyo3::{
    Bound, IntoPyObjectExt, Py, PyAny, PyResult, Python, pyclass, pymethods,
    types::{PyDict, PyDictMethods},
};
use radiate::{Loss, NoveltySearch, Regression};

#[pyclass]
#[derive(Clone)]
pub struct PyNoveltySearchFitnessBuilder {
    pub descriptor: ObjectValue,
    pub distance: String,
    pub k: usize,
    pub threshold: f32,
    pub archive_size: usize,
}

#[pymethods]
impl PyNoveltySearchFitnessBuilder {
    #[new]
    pub fn new(
        descriptor: Py<PyAny>,
        distance: String,
        k: usize,
        threshold: f32,
        archive_size: usize,
    ) -> Self {
        PyNoveltySearchFitnessBuilder {
            descriptor: ObjectValue { inner: descriptor },
            distance,
            k,
            threshold,
            archive_size,
        }
    }
}

#[derive(Clone)]
pub enum PyFitnessInner {
    Custom(ObjectValue),
    Regression(Regression),
    NoveltySearch(ObjectValue),
}

#[pyclass]
#[derive(Clone)]
pub struct PyFitnessFn {
    pub inner: PyFitnessInner,
}

#[pymethods]
impl PyFitnessFn {
    #[staticmethod]
    pub fn custom(fitness_fn: Py<PyAny>) -> Self {
        PyFitnessFn {
            inner: PyFitnessInner::Custom(ObjectValue { inner: fitness_fn }),
        }
    }

    #[staticmethod]
    pub fn regression(features: Vec<Vec<f32>>, targets: Vec<Vec<f32>>, loss: String) -> Self {
        let loss = match loss.as_str() {
            "mse" => Loss::MSE,
            "mae" => Loss::MAE,
            _ => panic!("Unsupported loss function: {}", loss),
        };

        PyFitnessFn {
            inner: PyFitnessInner::Regression(Regression::new((features, targets), loss)),
        }
    }

    #[staticmethod]
    pub fn novelty_search<'py>(
        py: Python<'py>,
        descriptor: Py<PyAny>,
        distance_fn: String,
        k: usize,
        threshold: f32,
        archive_size: usize,
    ) -> Self {
        let search = PyNoveltySearch::new(descriptor, k, threshold, archive_size, distance_fn);
        PyFitnessFn {
            inner: PyFitnessInner::NoveltySearch(ObjectValue {
                inner: search.into_py_any(py).unwrap(),
            }),
        }
    }
}

// pub(crate) const CUSTOM_PROBLEM: &str = "Custom";
// pub(crate) const REGRESSION_PROBLEM: &str = "Regression";
// pub(crate) const NOVELTY_SEARCH_PROBLEM: &str = "Novelty";

// #[pyclass]
// #[derive(Clone, Debug)]
// pub struct PyProblemBuilder {
//     pub name: String,
//     pub args: ObjectValue,
//     pub allowed_genes: Vec<PyGeneType>,
//     pub is_native: bool,
// }

// #[pymethods]
// impl PyProblemBuilder {
//     pub fn __repr__(&self) -> String {
//         format!(
//             "PyTestProblem(name='{}', args={:?}, allowed_genes={:?})",
//             self.name, self.args, self.allowed_genes
//         )
//     }

//     pub fn __str__(&self) -> String {
//         self.__repr__()
//     }

//     pub fn name(&self) -> String {
//         self.name.clone()
//     }

//     pub fn args<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
//         self.args.inner.bind(py).into_bound_py_any(py)
//     }

//     #[staticmethod]
//     pub fn custom<'py>(py: Python<'py>, fitness_fn: Py<PyAny>) -> Self {
//         let args = PyDict::new(py);
//         args.set_item("fitness_func", fitness_fn).unwrap();

//         PyProblemBuilder {
//             name: CUSTOM_PROBLEM.into(),
//             args: ObjectValue {
//                 inner: args.unbind().into_any(),
//             },
//             is_native: false,
//             allowed_genes: vec![
//                 PyGeneType::Float,
//                 PyGeneType::Int,
//                 PyGeneType::Char,
//                 PyGeneType::Bit,
//                 PyGeneType::Permutation,
//                 PyGeneType::Graph,
//                 PyGeneType::Tree,
//             ],
//         }
//     }

//     #[staticmethod]
//     pub fn regression<'py>(
//         py: Python<'py>,
//         features: Vec<Vec<f32>>,
//         targets: Vec<Vec<f32>>,
//         loss: String,
//     ) -> Self {
//         let args = PyDict::new(py);

//         args.set_item("features", features).unwrap();
//         args.set_item("targets", targets).unwrap();
//         args.set_item("loss", loss).unwrap();

//         PyProblemBuilder {
//             name: REGRESSION_PROBLEM.into(),
//             args: ObjectValue {
//                 inner: args.unbind().into_any(),
//             },
//             is_native: true,
//             allowed_genes: vec![PyGeneType::Graph, PyGeneType::Tree],
//         }
//     }

//     #[staticmethod]
//     pub fn custom_novelty_search<'py>(
//         py: Python<'py>,
//         distance: PyEngineInput,
//         descriptor: Py<PyAny>,
//         k: usize,
//         threshold: f32,
//         archive_size: usize,
//         is_native: bool,
//     ) -> Self {
//         let allowed_genes = distance
//             .allowed_genes
//             .clone()
//             .into_iter()
//             .collect::<Vec<PyGeneType>>();

//         let args = PyDict::new(py);

//         args.set_item("descriptor", descriptor).unwrap();
//         args.set_item("distance", distance).unwrap();
//         args.set_item("k", k).unwrap();
//         args.set_item("threshold", threshold).unwrap();
//         args.set_item("max_archive_size", archive_size).unwrap();

//         PyProblemBuilder {
//             name: NOVELTY_SEARCH_PROBLEM.into(),
//             args: ObjectValue {
//                 inner: args.unbind().into_any(),
//             },
//             is_native,
//             allowed_genes,
//         }
//     }

//     #[staticmethod]
//     pub fn novelty_search<'py>(
//         py: Python<'py>,
//         distance: PyEngineInput,
//         descriptor: Option<Py<PyAny>>,
//         k: usize,
//         threshold: f32,
//         archive_size: usize,
//         is_native: bool,
//     ) -> Self {
//         let allowed_genes = distance
//             .allowed_genes
//             .clone()
//             .into_iter()
//             .collect::<Vec<PyGeneType>>();

//         let args = PyDict::new(py);

//         if let Some(desc) = descriptor {
//             args.set_item("descriptor", desc).unwrap();
//         }

//         args.set_item("distance", distance).unwrap();
//         args.set_item("k", k).unwrap();
//         args.set_item("threshold", threshold).unwrap();
//         args.set_item("max_archive_size", archive_size).unwrap();

//         PyProblemBuilder {
//             name: NOVELTY_SEARCH_PROBLEM.into(),
//             args: ObjectValue {
//                 inner: args.unbind().into_any(),
//             },
//             is_native,
//             allowed_genes,
//         }
//     }
// }
