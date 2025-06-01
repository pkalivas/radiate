use crate::ObjectValue;
use pyo3::{pyclass, pymethods};

#[pyclass]
#[derive(Clone)]
pub struct PyFunc {
    pub func: ObjectValue,
}

#[pymethods]
impl PyFunc {
    #[new]
    pub fn new(func: ObjectValue) -> Self {
        Self { func }
    }
}
