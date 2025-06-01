use pyo3::{pyclass, pymethods};
use radiate::{Metric, MetricSet};
use std::time::Duration;

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

    pub fn get_metric(&self, name: String) -> Option<PyMetric> {
        self.metrics
            .get_from_string(name)
            .map(|metric| PyMetric(metric.clone()))
    }
}

impl Into<PyMetricSet> for MetricSet {
    fn into(self) -> PyMetricSet {
        PyMetricSet { metrics: self }
    }
}

#[pyclass]
#[derive(Clone)]
#[repr(transparent)]
pub struct PyMetric(pub Metric);

#[pymethods]
impl PyMetric {
    pub fn name(&self) -> String {
        self.0.name().to_string()
    }

    pub fn last_value(&self) -> f32 {
        self.0.last_value()
    }

    pub fn last_time(&self) -> Duration {
        self.0.last_time()
    }

    pub fn value_mean(&self) -> Option<f32> {
        self.0.value_mean()
    }

    pub fn value_stddev(&self) -> Option<f32> {
        self.0.value_std_dev()
    }

    pub fn variance(&self) -> Option<f32> {
        self.0.value_variance()
    }

    pub fn value_min(&self) -> Option<f32> {
        self.0.value_min()
    }

    pub fn value_max(&self) -> Option<f32> {
        self.0.value_max()
    }

    pub fn value_count(&self) -> i32 {
        self.0.count()
    }

    pub fn time_sum(&self) -> Option<Duration> {
        self.0.time_sum()
    }

    pub fn time_mean(&self) -> Option<Duration> {
        self.0.time_mean()
    }

    pub fn time_std_dev(&self) -> Option<Duration> {
        self.0.time_std_dev()
    }

    pub fn time_min(&self) -> Option<Duration> {
        self.0.time_min()
    }

    pub fn time_max(&self) -> Option<Duration> {
        self.0.time_max()
    }

    pub fn __repr__(&self) -> String {
        format!("{:?}", &self.0)
    }

    pub fn __str__(&self) -> String {
        format!("{:?}", &self.0)
    }
}

impl Into<PyMetric> for Metric {
    fn into(self) -> PyMetric {
        PyMetric(self)
    }
}
