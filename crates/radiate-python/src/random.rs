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
}
