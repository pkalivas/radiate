use pyo3::{pyclass, pymethods};
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
}
