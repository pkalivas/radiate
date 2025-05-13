use crate::PyMetricSet;
use pyo3::{
    Bound, IntoPyObjectExt, Py, PyAny, PyResult, Python, pyclass, pymethods, types::PyList,
};

#[pyclass(unsendable)]
pub struct PyGeneration {
    pub index: usize,
    pub score: Py<PyList>,
    pub value: Py<PyAny>,
    pub metrics: PyMetricSet,
}

#[pymethods]
impl PyGeneration {
    #[new]
    #[pyo3(signature = (index, score, value, metrics))]
    pub fn new(index: usize, score: Py<PyList>, value: Py<PyAny>, metrics: PyMetricSet) -> Self {
        Self {
            index,
            score,
            value,
            metrics,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn score<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.score.as_any().into_bound_py_any(py)
    }

    pub fn value<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.value.as_any().into_bound_py_any(py)
    }

    pub fn __repr__(&self, py: Python) -> PyResult<String> {
        let score = self.score(py)?;
        let value = self.value(py)?;

        Ok(format!(
            "Generation(\n\tindex={},\n\tscore={},\n\tvalue={},\n\t metrics={})",
            self.index,
            score,
            value,
            self.metrics.__repr__()
        ))
    }
}
