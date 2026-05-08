use crate::PyExpr;
use crate::object::Wrap;
use pyo3::exceptions::PyValueError;
use pyo3::types::PyDict;
use pyo3::{IntoPyObject, PyErr, PyResult, Python};
use pyo3::{intern, prelude::*};
use pyo3::{pyclass, pymethods};
use radiate::{AnyValue, Evaluate, Metric, MetricSet, MetricUpdate};
use radiate_error::radiate_py_bail;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[pyclass(skip_from_py_object)]
#[derive(Clone, Deserialize, Serialize)]
#[repr(transparent)]
pub struct PyMetricSet {
    inner: MetricSet,
}

#[pymethods]
impl PyMetricSet {
    #[new]
    #[pyo3(signature = (metrics=None))]
    pub fn new(metrics: Option<Wrap<AnyValue<'_>>>) -> PyResult<Self> {
        if let Some(metrics) = metrics {
            let mut metric_set = MetricSet::new();
            if let AnyValue::Struct(pairs) = metrics.0.into_static() {
                for (fld, val) in pairs.into_iter() {
                    let name = fld.name().to_string();
                    let metric_update = MetricUpdate::try_from(val)?;
                    metric_set.upsert((radiate_utils::intern!(name), metric_update));
                }
            } else {
                radiate_py_bail!("Metric: Expected a struct of metrics, but got a different type.");
            }

            return Ok(PyMetricSet { inner: metric_set });
        }

        Ok(PyMetricSet {
            inner: MetricSet::new(),
        })
    }

    pub fn upsert(&mut self, name: &str, update: Wrap<AnyValue<'_>>) -> PyResult<()> {
        let interned_name = radiate_utils::intern!(name);
        let metric_update = MetricUpdate::try_from(update.0)?;
        self.inner.upsert((interned_name, metric_update));
        Ok(())
    }

    pub fn __repr__(&self) -> String {
        let summary = self.inner.summary();
        format!(
            "MetricSet(metrics={}, updates={})",
            summary.metrics, summary.updates
        )
    }

    pub fn __dict__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        let rows = self.to_rows(py)?;
        for row in rows {
            dict.set_item(row.get_item("name")?, row)?;
        }

        Ok(dict)
    }

    pub fn __getitem__<'py>(
        &self,
        py: Python<'py>,
        key: String,
    ) -> PyResult<Option<Bound<'py, PyMetric>>> {
        if !self.inner.contains_key(&key) {
            return Ok(None);
        }

        self.inner
            .get_from_string(key)
            .map(|metric| Wrap(metric).into_pyobject(py))
            .transpose()
            .map_err(|e| {
                PyValueError::new_err(format!(
                    "{e} Unknown error occurred while converting Metric to Python dict."
                ))
            })
    }

    pub fn __contains__(&self, key: String) -> bool {
        self.inner.contains_key(&key)
    }

    pub fn __len__(&self) -> usize {
        self.inner.len()
    }

    pub fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner).map_err(|e| {
            PyValueError::new_err(format!(
                "{e} Unknown error occurred while converting MetricSet to JSON."
            ))
        })
    }

    pub fn dashboard(&self) -> String {
        format!("{}", &self.inner)
    }

    pub fn keys(&self) -> Vec<&'static str> {
        self.inner.keys()
    }

    pub fn project<'py>(&self, py: Python<'py>, expr: &mut PyExpr) -> PyResult<Bound<'py, PyAny>> {
        let result = expr.inner_mut().eval(&self.inner).unwrap_or(AnyValue::Null);
        Wrap(result).into_pyobject(py)
    }

    pub fn values_by_tag<'py>(
        &self,
        py: Python<'py>,
        tag: String,
    ) -> PyResult<Vec<Bound<'py, PyMetric>>> {
        let mut vec = Vec::new();
        for metric in self.inner.iter_tagged(tag.into()) {
            vec.push(Wrap(metric.1).into_pyobject(py)?);
        }

        Ok(vec)
    }

    pub fn values<'py>(&self, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyMetric>>> {
        let mut vec = Vec::with_capacity(self.inner.len());
        for metric in self.inner.iter() {
            vec.push(Wrap(metric.1).into_pyobject(py)?);
        }

        Ok(vec)
    }

    pub fn items<'py>(&self, py: Python<'py>) -> PyResult<Vec<(String, Bound<'py, PyMetric>)>> {
        let mut vec = Vec::with_capacity(self.inner.len());
        for (key, metric) in self.inner.iter() {
            vec.push(((*key).to_string(), Wrap(metric).into_pyobject(py)?));
        }

        Ok(vec)
    }

    pub fn to_rows<'py>(&self, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyDict>>> {
        let mut out = Vec::with_capacity(self.inner.len() * 3);

        for (_, m) in self.inner.iter() {
            out.push(PyMetric::from(m.clone()).to_dict(py)?);
        }

        Ok(out)
    }

    pub fn to_pandas<'py>(&self, py: Python<'py>) -> PyResult<pyo3::Bound<'py, PyAny>> {
        let rows = self.to_rows(py)?;
        let pd = py.import("pandas")?;
        let df = pd.getattr("DataFrame")?.call1((rows,))?;
        Ok(df)
    }

    pub fn to_polars<'py>(&self, py: Python<'py>, lazy: bool) -> PyResult<pyo3::Bound<'py, PyAny>> {
        let rows = self.to_rows(py)?;
        let pl = py.import("polars")?;
        let df = if lazy {
            pl.getattr("LazyFrame")?.call1((rows,))?
        } else {
            pl.getattr("DataFrame")?.call1((rows,))?
        };
        Ok(df)
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

#[pyclass(skip_from_py_object)]
#[derive(Clone)]
#[repr(transparent)]
pub struct PyMetric {
    inner: Metric,
}

impl From<Metric> for PyMetric {
    fn from(m: Metric) -> Self {
        Self { inner: m }
    }
}

#[pymethods]
impl PyMetric {
    #[staticmethod]
    #[pyo3(signature = (name, values=None))]
    pub fn new(name: String, values: Option<Wrap<AnyValue<'_>>>) -> PyResult<Self> {
        let mut metric = Metric::new(radiate_utils::intern!(name));
        if let Some(values) = values {
            let metric_values = values.0.into_static();
            let metric_update = MetricUpdate::try_from(metric_values)?;

            metric.apply_update(metric_update);
        }

        Ok(PyMetric { inner: metric })
    }

    pub fn __repr__(&self) -> String {
        format!("PyMetric(name='{}')", self.inner.name())
    }

    #[getter]
    pub fn name(&self) -> &str {
        self.inner.name()
    }

    #[getter]
    pub fn tags(&self) -> Vec<&'static str> {
        self.inner
            .tags()
            .iter()
            .map(|tag| radiate_utils::intern!(tag.as_str().to_string().to_lowercase()))
            .collect()
    }

    #[getter]
    pub fn version(&self) -> u64 {
        self.inner.version()
    }

    #[getter]
    pub fn update_count(&self) -> usize {
        self.inner.update_count()
    }

    // --- value stats ---
    #[getter]
    pub fn value_last(&self) -> f32 {
        self.inner.last_value()
    }

    #[getter]
    pub fn value_sum(&self) -> f32 {
        self.inner.sum()
    }

    #[getter]
    pub fn value_mean(&self) -> f32 {
        self.inner.mean()
    }

    #[getter]
    pub fn value_stddev(&self) -> f32 {
        self.inner.stddev()
    }

    #[getter]
    pub fn value_variance(&self) -> f32 {
        self.inner.var()
    }

    #[getter]
    pub fn value_skewness(&self) -> f32 {
        self.inner.skew()
    }

    #[getter]
    pub fn value_min(&self) -> f32 {
        self.inner.min()
    }

    #[getter]
    pub fn value_max(&self) -> f32 {
        self.inner.max()
    }

    #[getter]
    pub fn value_count(&self) -> u32 {
        self.inner.count()
    }

    // --- time stats (seconds as float) ---
    #[getter]
    pub fn time_last(&self) -> Duration {
        self.inner
            .times()
            .map(|time| time.last())
            .unwrap_or_default()
    }

    #[getter]
    pub fn time_sum(&self) -> Option<Duration> {
        self.inner.times().map(|time| time.sum())
    }

    #[getter]
    pub fn time_mean(&self) -> Option<Duration> {
        self.inner.times().map(|time| time.mean())
    }

    #[getter]
    pub fn time_stddev(&self) -> Option<Duration> {
        self.inner.times().map(|time| time.stddev())
    }

    #[getter]
    pub fn time_min(&self) -> Option<Duration> {
        self.inner.times().map(|time| time.min())
    }

    #[getter]
    pub fn time_max(&self) -> Option<Duration> {
        self.inner.times().map(|time| time.max())
    }

    #[getter]
    pub fn time_variance(&self) -> Option<Duration> {
        self.inner.times().map(|time| time.var())
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let d = PyDict::new(py);
        d.set_item(intern!(py, "name"), self.inner.name().to_string())?;

        d.set_item(intern!(py, "last"), self.value_last())?;
        d.set_item(intern!(py, "sum"), self.value_sum())?;
        d.set_item(intern!(py, "mean"), self.value_mean())?;
        d.set_item(intern!(py, "stddev"), self.value_stddev())?;
        d.set_item(intern!(py, "var"), self.value_variance())?;
        d.set_item(intern!(py, "skew"), self.value_skewness())?;
        d.set_item(intern!(py, "min"), self.value_min())?;
        d.set_item(intern!(py, "max"), self.value_max())?;
        d.set_item(intern!(py, "count"), self.value_count())?;

        d.set_item(intern!(py, "time_sum"), self.time_sum())?;
        d.set_item(intern!(py, "time_mean"), self.time_mean())?;
        d.set_item(intern!(py, "time_stddev"), self.time_stddev())?;
        d.set_item(intern!(py, "time_min"), self.time_min())?;
        d.set_item(intern!(py, "time_max"), self.time_max())?;
        d.set_item(intern!(py, "time_var"), self.time_variance())?;

        d.set_item(intern!(py, "version"), self.version())?;
        d.set_item(intern!(py, "update_count"), self.update_count())?;

        d.set_item(intern!(py, "tags"), self.tags())?;

        Ok(d)
    }
}

impl<'py> IntoPyObject<'py> for Wrap<Metric> {
    type Target = PyMetric;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Bound::new(py, PyMetric::from(self.0))
    }
}

impl<'py> IntoPyObject<'py> for Wrap<&Metric> {
    type Target = PyMetric;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Bound::new(py, PyMetric::from(self.0.clone()))
    }
}
