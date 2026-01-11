use super::PyGenotype;
use pyo3::{PyResult, pyclass, pymethods};

#[pyclass(unsendable)]
#[derive(Clone)]
pub struct PyFrontValue {
    pub(crate) genotype: PyGenotype,
    pub(crate) score: Option<Vec<f32>>,
}

#[pymethods]
impl PyFrontValue {
    pub fn genotype(&self) -> PyGenotype {
        self.genotype.clone()
    }

    pub fn score(&self) -> Option<Vec<f32>> {
        self.score.clone()
    }
}

#[pyclass(unsendable)]
pub struct PyFront {
    pub(crate) inner: Vec<PyFrontValue>,
}

impl PyFront {
    pub fn new(inner: Vec<PyFrontValue>) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyFront {
    pub fn __len__(&self) -> PyResult<usize> {
        Ok(self.inner.len())
    }

    pub fn __getitem__(&self, index: isize) -> PyResult<PyFrontValue> {
        let len = self.inner.len() as isize;
        let idx = if index < 0 { len + index } else { index };

        if idx < 0 || idx >= len {
            Err(pyo3::exceptions::PyIndexError::new_err(
                "Index out of range",
            ))
        } else {
            Ok(self.inner[idx as usize].clone())
        }
    }

    pub fn values(&self) -> Vec<PyFrontValue> {
        self.inner.iter().map(|obj| obj.clone()).collect()
    }
}
