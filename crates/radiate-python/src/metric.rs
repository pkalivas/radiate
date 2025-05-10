use pyo3::{pyclass, pymethods};
use radiate::MetricSet;

#[pyclass]
#[derive(Clone)]
#[repr(transparent)]
pub struct PyMetricSet {
    pub metrics: MetricSet,
}

#[pymethods]
impl PyMetricSet {
    pub fn __repr__(&self) -> String {
        format!("{:?}", &self.metrics)
    }
}

impl Into<PyMetricSet> for MetricSet {
    fn into(self) -> PyMetricSet {
        PyMetricSet { metrics: self }
    }
}
