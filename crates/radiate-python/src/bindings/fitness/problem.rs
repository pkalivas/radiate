use crate::{ObjectValue, PyNoveltySearch};
use pyo3::{IntoPyObjectExt, Py, PyAny, Python, pyclass, pymethods};
use radiate::{Loss, Regression};

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
