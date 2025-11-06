use crate::{PyAnyObject, PyNoveltySearch};
use pyo3::{IntoPyObjectExt, Py, PyAny, Python, pyclass, pymethods};
use radiate::{Loss, RadiateResult, Regression};

#[derive(Clone)]
pub enum PyFitnessInner {
    Custom(PyAnyObject, bool), // bool indicates if batch
    Regression(Regression, bool),
    NoveltySearch(PyAnyObject, bool),
}

impl PyFitnessInner {
    pub fn get_fitness_fn(&self) -> RadiateResult<Py<PyAny>> {
        match self {
            PyFitnessInner::Custom(func, _) => Ok(func.clone().inner),
            PyFitnessInner::Regression(_, _) => Err(radiate::error::radiate_err!(
                "Regression fitness functions do not have a direct PyAny representation"
            )),
            PyFitnessInner::NoveltySearch(func, _) => Ok(func.clone().inner),
        }
    }

    pub fn is_batch(&self) -> bool {
        match self {
            PyFitnessInner::Custom(_, is_batch) => *is_batch,
            PyFitnessInner::Regression(_, is_batch) => *is_batch,
            PyFitnessInner::NoveltySearch(_, is_batch) => *is_batch,
        }
    }
}

#[pyclass]
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
    pub fn regression(
        features: Vec<Vec<f32>>,
        targets: Vec<Vec<f32>>,
        loss: String,
        is_batch: bool,
    ) -> Self {
        let loss = match loss.as_str() {
            "mse" => Loss::MSE,
            "mae" => Loss::MAE,
            "cross_entropy" => Loss::CrossEntropy,
            "diff" => Loss::Diff,
            _ => panic!("Unsupported loss function: {}", loss),
        };

        PyFitnessFn {
            inner: PyFitnessInner::Regression(Regression::new((features, targets), loss), is_batch),
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
