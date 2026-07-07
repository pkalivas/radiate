use crate::bindings::datatype::{FloatMatrixPair, extract_regression_pair};
use crate::{PyAnyObject, PyNoveltySearch};
use pyo3::{
    Bound, IntoPyObjectExt, Py, PyAny, PyResult, Python, exceptions::PyValueError, pyclass,
    pymethods,
};
use radiate::{Loss, RadiateResult, Regression};

#[derive(Clone)]
pub enum PyFitnessInner {
    Custom(PyAnyObject, bool), // bool indicates if batch
    Regression32(Regression<f32>, bool),
    Regression64(Regression<f64>, bool),
    NoveltySearch(PyAnyObject, bool),
}

impl PyFitnessInner {
    pub fn get_fitness_fn(&self) -> RadiateResult<Py<PyAny>> {
        match self {
            PyFitnessInner::Custom(func, _) => Ok(func.clone().inner),
            PyFitnessInner::Regression32(_, _) | PyFitnessInner::Regression64(_, _) => {
                Err(radiate::error::radiate_err!(
                    "Regression fitness functions do not have a direct PyAny representation"
                ))
            }
            PyFitnessInner::NoveltySearch(func, _) => Ok(func.clone().inner),
        }
    }

    pub fn is_batch(&self) -> bool {
        match self {
            PyFitnessInner::Custom(_, is_batch) => *is_batch,
            PyFitnessInner::Regression32(_, is_batch) => *is_batch,
            PyFitnessInner::Regression64(_, is_batch) => *is_batch,
            PyFitnessInner::NoveltySearch(_, is_batch) => *is_batch,
        }
    }
}

#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct PyFitnessFn {
    pub inner: PyFitnessInner,
}

#[pymethods]
impl PyFitnessFn {
    #[staticmethod]
    pub fn custom(fitness_fn: Py<PyAny>, is_batch: bool) -> Self {
        PyFitnessFn {
            inner: PyFitnessInner::Custom(PyAnyObject { inner: fitness_fn }, is_batch),
        }
    }

    #[staticmethod]
    pub fn regression<'py>(
        py: Python<'py>,
        features: &Bound<'py, PyAny>,
        targets: &Bound<'py, PyAny>,
        loss: String,
        is_batch: bool,
    ) -> PyResult<Self> {
        let loss = match loss.to_lowercase().as_str() {
            crate::constants::loss_functions::MSE_LOSS => Loss::MSE,
            crate::constants::loss_functions::MAE_LOSS => Loss::MAE,
            crate::constants::loss_functions::CROSS_ENTROPY_LOSS => Loss::XEnt,
            crate::constants::loss_functions::DIFF_LOSS => Loss::Diff,
            _ => {
                return Err(PyValueError::new_err(format!(
                    "Unsupported loss function: {}",
                    loss
                )));
            }
        };

        // Width is decided once, for the pair, in one place (datatype::array) —
        // f32 only when both features and targets are already f32 NumPy arrays;
        // f64 otherwise (f64 arrays, plain lists, or a mismatched pair).
        let inner = match extract_regression_pair(py, features, targets)? {
            FloatMatrixPair::F32 { features, targets } => {
                PyFitnessInner::Regression32(Regression::new((features, targets), loss), is_batch)
            }
            FloatMatrixPair::F64 { features, targets } => {
                PyFitnessInner::Regression64(Regression::new((features, targets), loss), is_batch)
            }
        };

        Ok(PyFitnessFn { inner })
    }

    #[staticmethod]
    pub fn novelty_search<'py>(
        py: Python<'py>,
        descriptor: Py<PyAny>,
        distance_fn: String,
        k: usize,
        threshold: f32,
        archive_size: usize,
        is_batch: bool,
    ) -> Self {
        let search = PyNoveltySearch::new(descriptor, k, threshold, archive_size, distance_fn);
        PyFitnessFn {
            inner: PyFitnessInner::NoveltySearch(
                PyAnyObject {
                    inner: search.into_py_any(py).unwrap(),
                },
                is_batch,
            ),
        }
    }
}
