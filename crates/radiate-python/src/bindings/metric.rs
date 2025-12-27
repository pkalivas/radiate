use crate::object::Wrap;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::{IntoPyObject, PyErr, PyResult, Python};
use pyo3::{pyclass, pymethods};
use radiate::stats::TagKind;
use radiate::{Metric, MetricSet};
use serde::{Deserialize, Serialize};

#[pyclass]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PyTagKind {
    Selector,
    Alterer,
    Mutator,
    Crossover,
    Species,
    Failure,
    Age,
    Front,
    Derived,
    Other,
    Statistic,
    Time,
    Distribution,
    Score,
}

impl Into<TagKind> for PyTagKind {
    fn into(self) -> TagKind {
        match self {
            PyTagKind::Selector => TagKind::Selector,
            PyTagKind::Alterer => TagKind::Alterer,
            PyTagKind::Mutator => TagKind::Mutator,
            PyTagKind::Crossover => TagKind::Crossover,
            PyTagKind::Species => TagKind::Species,
            PyTagKind::Failure => TagKind::Failure,
            PyTagKind::Age => TagKind::Age,
            PyTagKind::Front => TagKind::Front,
            PyTagKind::Derived => TagKind::Derived,
            PyTagKind::Other => TagKind::Other,
            PyTagKind::Statistic => TagKind::Statistic,
            PyTagKind::Time => TagKind::Time,
            PyTagKind::Distribution => TagKind::Distribution,
            PyTagKind::Score => TagKind::Score,
        }
    }
}

#[pyclass]
#[derive(Clone, Deserialize, Serialize)]
#[repr(transparent)]
pub struct PyMetricSet {
    inner: MetricSet,
}

#[pymethods]
impl PyMetricSet {
    pub fn __repr__(&self) -> String {
        let summary = self.inner.summary();
        format!(
            "MetricSet[metrics={}, updates={}]",
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

    pub fn values_by_tag<'py>(
        &self,
        py: Python<'py>,
        tag: PyTagKind,
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

    pub fn to_polars<'py>(&self, py: Python<'py>) -> PyResult<pyo3::Bound<'py, PyAny>> {
        let rows = self.to_rows(py)?;
        let pl = py.import("polars")?;
        let df = pl.getattr("DataFrame")?.call1((rows,))?;
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

#[pyclass]
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
    pub fn __repr__(&self) -> String {
        format!(
            "PyMetric(name='{}', count={})",
            self.inner.name(),
            self.inner.count()
        )
    }

    pub fn __dict__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        self.to_dict(py)
    }

    #[getter]
    pub fn name(&self) -> &str {
        self.inner.name()
    }

    // --- value stats ---
    #[getter]
    pub fn value_last(&self) -> f32 {
        self.inner.last_value()
    }

    #[getter]
    pub fn value_sum(&self) -> Option<f32> {
        self.inner.value_sum()
    }

    #[getter]
    pub fn value_mean(&self) -> Option<f32> {
        self.inner.value_mean()
    }

    #[getter]
    pub fn value_stddev(&self) -> Option<f32> {
        self.inner.value_std_dev()
    }

    #[getter]
    pub fn value_variance(&self) -> Option<f32> {
        self.inner.value_variance()
    }

    #[getter]
    pub fn value_skewness(&self) -> Option<f32> {
        self.inner.value_skewness()
    }

    #[getter]
    pub fn value_min(&self) -> Option<f32> {
        self.inner.value_min()
    }

    #[getter]
    pub fn value_max(&self) -> Option<f32> {
        self.inner.value_max()
    }

    #[getter]
    pub fn value_count(&self) -> i32 {
        self.inner.count()
    }

    // --- time stats (seconds as float) ---
    #[getter]
    pub fn time_last(&self) -> f64 {
        self.inner.last_time().as_secs_f64()
    }

    #[getter]
    pub fn time_sum(&self) -> Option<f64> {
        self.inner.time_sum().map(|d| d.as_secs_f64())
    }

    #[getter]
    pub fn time_mean(&self) -> Option<f64> {
        self.inner.time_mean().map(|d| d.as_secs_f64())
    }

    #[getter]
    pub fn time_stddev(&self) -> Option<f64> {
        self.inner.time_std_dev().map(|d| d.as_secs_f64())
    }

    #[getter]
    pub fn time_min(&self) -> Option<f64> {
        self.inner.time_min().map(|d| d.as_secs_f64())
    }

    #[getter]
    pub fn time_max(&self) -> Option<f64> {
        self.inner.time_max().map(|d| d.as_secs_f64())
    }

    #[getter]
    pub fn time_variance(&self) -> Option<f64> {
        self.inner.time_variance().map(|d| d.as_secs_f64())
    }

    /// Convert to a dict (nice for DataFrame construction / JSON dumps).
    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let d = PyDict::new(py);
        d.set_item("name", self.inner.name().to_string())?;

        d.set_item("last", self.value_last())?;
        d.set_item("sum", self.value_sum())?;
        d.set_item("mean", self.value_mean())?;
        d.set_item("stddev", self.value_stddev())?;
        d.set_item("var", self.value_variance())?;
        d.set_item("skew", self.value_skewness())?;
        d.set_item("min", self.value_min())?;
        d.set_item("max", self.value_max())?;
        d.set_item("count", self.value_count())?;

        d.set_item("time_sum", self.time_sum())?;
        d.set_item("time_mean", self.time_mean())?;
        d.set_item("time_stddev", self.time_stddev())?;
        d.set_item("time_min", self.time_min())?;
        d.set_item("time_max", self.time_max())?;
        d.set_item("time_var", self.time_variance())?;

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
