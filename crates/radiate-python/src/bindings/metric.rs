use crate::conversion::Wrap;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::{IntoPyObject, PyErr, PyResult, Python};
use radiate::{Metric, MetricSet};

#[pyclass]
#[derive(Clone)]
#[repr(transparent)]
pub struct PyMetricSet {
    inner: MetricSet,
}

#[pymethods]
impl PyMetricSet {
    pub fn __repr__(&self) -> String {
        format!("{:?}", &self.inner)
    }

    pub fn get_metric<'py>(
        &self,
        py: Python<'py>,
        name: String,
    ) -> PyResult<Option<Bound<'py, PyDict>>> {
        self.inner
            .get_from_string(name)
            .map(|metric| Wrap(metric).into_pyobject(py))
            .transpose()
            .map_err(|e| {
                PyValueError::new_err(format!(
                    "{e}\n\nHint: Try setting `strict=False` to allow passing data with mixed types."
                ))
            })
    }
}

impl From<MetricSet> for PyMetricSet {
    fn from(metric_set: MetricSet) -> Self {
        PyMetricSet { inner: metric_set }
    }
}

impl<'py> IntoPyObject<'py> for Wrap<&MetricSet> {
    type Target = PyMetricSet;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let metric_set = self.0.clone();
        Bound::new(py, PyMetricSet::from(metric_set))
    }
}

impl<'py> IntoPyObject<'py> for Wrap<MetricSet> {
    type Target = PyMetricSet;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let metric_set = self.0;
        Bound::new(py, PyMetricSet::from(metric_set))
    }
}

impl<'py> IntoPyObject<'py> for Wrap<Metric> {
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        metric_to_py_dict(py, &self.0).map_err(|e| {
            PyValueError::new_err(format!(
                "{e}\n\nHint: Try setting `strict=False` to allow passing data with mixed types."
            ))
        })
    }
}

impl<'py> IntoPyObject<'py> for Wrap<&Metric> {
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        metric_to_py_dict(py, &self.0).map_err(|e| {
            PyValueError::new_err(format!(
                "{e}\n\nHint: Try setting `strict=False` to allow passing data with mixed types."
            ))
        })
    }
}

fn metric_to_py_dict<'py, 'a>(py: Python<'py>, metric: &Metric) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);

    dict.set_item("value_last", metric.last_value())?;
    dict.set_item("value_mean", metric.value_mean())?;
    dict.set_item("value_stddev", metric.value_std_dev())?;
    dict.set_item("value_variance", metric.value_variance())?;
    dict.set_item("value_skewness", metric.value_skewness())?;
    dict.set_item("value_min", metric.value_min())?;
    dict.set_item("value_max", metric.value_max())?;
    dict.set_item("value_count", metric.count())?;

    dict.set_item("sequence_last", metric.last_sequence())?;
    dict.set_item("sequence_mean", metric.distribution_mean())?;
    dict.set_item("sequence_stddev", metric.distribution_std_dev())?;
    dict.set_item("sequence_min", metric.distribution_min())?;
    dict.set_item("sequence_max", metric.distribution_max())?;
    dict.set_item("sequence_variance", metric.distribution_variance())?;
    dict.set_item("sequence_skewness", metric.distribution_skewness())?;
    dict.set_item("sequence_kurtosis", metric.distribution_kurtosis())?;

    dict.set_item("time_last", metric.last_time())?;
    dict.set_item("time_sum", metric.time_sum())?;
    dict.set_item("time_mean", metric.time_mean())?;
    dict.set_item("time_std_dev", metric.time_std_dev())?;
    dict.set_item("time_min", metric.time_min())?;
    dict.set_item("time_max", metric.time_max())?;
    dict.set_item("time_variance", metric.time_variance())?;

    let result = PyDict::new(py);
    result.set_item("name", metric.name())?;
    result.set_item("type", metric.metric_type())?;
    result.set_item("metrics", dict)?;

    Ok(result)
}
