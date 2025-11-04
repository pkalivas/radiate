use pyo3::{Py, PyAny, pyclass, pymethods};
use radiate::random_provider;

#[pyclass]
pub struct PyRandomProvider;

#[pymethods]
impl PyRandomProvider {
    #[staticmethod]
    pub fn set_seed(seed: u64) {
        random_provider::set_seed(seed);
    }

    #[staticmethod]
    pub fn random_int(min: i64, max: i64) -> i64 {
        random_provider::range(min..max)
    }

    #[staticmethod]
    pub fn random_float(min: f64, max: f64) -> f64 {
        random_provider::range(min..max)
    }

    #[staticmethod]
    pub fn random_bool(prob: f64) -> bool {
        random_provider::bool(prob as f32)
    }

    #[staticmethod]
    pub fn sample(mut data: Vec<Py<PyAny>>, count: usize) -> Vec<Py<PyAny>> {
        random_provider::shuffle(&mut data);
        data.into_iter().take(count).collect()
    }

    #[staticmethod]
    pub fn choose(data: Vec<Py<PyAny>>) -> Py<PyAny> {
        Self::sample(data, 1).into_iter().next().unwrap()
    }
}
