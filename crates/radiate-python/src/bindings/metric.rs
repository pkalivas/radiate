use crate::object::Wrap;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::{IntoPyObject, PyErr, PyResult, Python};
use pyo3::{pyclass, pymethods};
use radiate::{Metric, MetricSet};
use serde::{Deserialize, Serialize};

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
        let rows = self.to_rows(py, true)?;
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

    /// Columns:
    ///   name, scope, rollup, kind, count, mean, min, max, std, total, entropy,
    ///   time_mean_ns, time_min_ns, time_max_ns, time_std_ns, time_sum_ns,
    ///   seq_mean, seq_min, seq_max, seq_std, seq_var, seq_skew, seq_kurt,
    ///   seq_last   (list[float] or None)
    #[pyo3(signature = (include_last_sequence=false))]
    pub fn to_rows<'py>(
        &self,
        py: Python<'py>,
        include_last_sequence: bool,
    ) -> PyResult<Vec<Bound<'py, PyDict>>> {
        let mut out: Vec<Bound<'py, PyDict>> = Vec::with_capacity(self.inner.len() * 3);

        for (_, m) in self.inner.iter() {
            let base_name = m.name().to_string();
            let scope = format!("{:?}", m.scope());
            let rollup = format!("{:?}", m.rollup());

            if let Some(stat) = m.statistic() {
                let row = PyDict::new(py);
                row.set_item("name", &base_name)?;
                row.set_item("scope", &scope)?;
                row.set_item("rollup", &rollup)?;
                row.set_item("kind", "value")?;

                row.set_item("count", stat.count())?;
                row.set_item("mean", stat.mean())?;
                row.set_item("min", stat.min())?;
                row.set_item("max", stat.max())?;
                row.set_item("std", stat.std_dev())?;
                row.set_item("total", stat.sum())?;

                // keep time/seq fields present but None so schema is consistent
                row.set_item("entropy", None::<f32>)?;
                row.set_item("time_mean_ns", None::<i128>)?;
                row.set_item("time_min_ns", None::<i128>)?;
                row.set_item("time_max_ns", None::<i128>)?;
                row.set_item("time_std_ns", None::<i128>)?;
                row.set_item("time_sum_ns", None::<i128>)?;
                row.set_item("seq_mean", None::<f32>)?;
                row.set_item("seq_min", None::<f32>)?;
                row.set_item("seq_max", None::<f32>)?;
                row.set_item("seq_std", None::<f32>)?;
                row.set_item("seq_var", None::<f32>)?;
                row.set_item("seq_skew", None::<f32>)?;
                row.set_item("seq_kurt", None::<f32>)?;
                row.set_item("seq_last", py.None())?;

                out.push(row);
            }

            if let Some(t) = m.time_statistic() {
                let to_ns = |d: std::time::Duration| -> i128 { d.as_nanos() as i128 };

                let row = PyDict::new(py);
                row.set_item("name", &base_name)?;
                row.set_item("scope", &scope)?;
                row.set_item("rollup", &rollup)?;
                row.set_item("kind", "time")?;

                row.set_item("count", t.count())?;
                row.set_item("mean", None::<f32>)?;
                row.set_item("min", None::<f32>)?;
                row.set_item("max", None::<f32>)?;
                row.set_item("std", None::<f32>)?;
                row.set_item("total", None::<f32>)?;
                row.set_item("entropy", None::<f32>)?;

                row.set_item("time_mean_ns", Some(to_ns(t.mean())))?;
                row.set_item("time_min_ns", Some(to_ns(t.min())))?;
                row.set_item("time_max_ns", Some(to_ns(t.max())))?;
                row.set_item("time_std_ns", Some(to_ns(t.standard_deviation())))?;
                row.set_item("time_sum_ns", Some(to_ns(t.sum())))?;

                row.set_item("seq_mean", None::<f32>)?;
                row.set_item("seq_min", None::<f32>)?;
                row.set_item("seq_max", None::<f32>)?;
                row.set_item("seq_std", None::<f32>)?;
                row.set_item("seq_var", None::<f32>)?;
                row.set_item("seq_skew", None::<f32>)?;
                row.set_item("seq_kurt", None::<f32>)?;
                row.set_item("seq_last", py.None())?;

                out.push(row);
            }

            // ----- dist row -----
            if let Some(d) = m.distribution() {
                let row = PyDict::new(py);
                row.set_item("name", &base_name)?;
                row.set_item("scope", &scope)?;
                row.set_item("rollup", &rollup)?;
                row.set_item("kind", "dist")?;

                row.set_item("count", d.count())?;
                row.set_item("mean", Some(d.mean()))?;
                row.set_item("min", Some(d.min()))?;
                row.set_item("max", Some(d.max()))?;
                row.set_item("std", Some(d.standard_deviation()))?;
                row.set_item("total", None::<f32>)?;
                row.set_item("entropy", Some(d.entropy()))?;

                row.set_item("time_mean_ns", None::<i128>)?;
                row.set_item("time_min_ns", None::<i128>)?;
                row.set_item("time_max_ns", None::<i128>)?;
                row.set_item("time_std_ns", None::<i128>)?;
                row.set_item("time_sum_ns", None::<i128>)?;

                row.set_item("seq_mean", Some(d.mean()))?;
                row.set_item("seq_min", Some(d.min()))?;
                row.set_item("seq_max", Some(d.max()))?;
                row.set_item("seq_std", Some(d.standard_deviation()))?;
                row.set_item("seq_var", Some(d.variance()))?;
                row.set_item("seq_skew", Some(d.skewness()))?;
                row.set_item("seq_kurt", Some(d.kurtosis()))?;
                if include_last_sequence {
                    row.set_item("seq_last", m.last_sequence().cloned())?;
                } else {
                    row.set_item("seq_last", py.None())?;
                }

                out.push(row);
            }
        }
        Ok(out)
    }

    #[pyo3(signature = (include_last_sequence=false))]
    pub fn to_pandas<'py>(
        &self,
        py: Python<'py>,
        include_last_sequence: bool,
    ) -> PyResult<pyo3::Bound<'py, PyAny>> {
        let rows = self.to_rows(py, include_last_sequence)?;
        let pd = py.import("pandas")?;
        let df = pd.getattr("DataFrame")?.call1((rows,))?;
        Ok(df)
    }

    #[pyo3(signature = (include_last_sequence=false))]
    pub fn to_polars<'py>(
        &self,
        py: Python<'py>,
        include_last_sequence: bool,
    ) -> PyResult<pyo3::Bound<'py, PyAny>> {
        let rows = self.to_rows(py, include_last_sequence)?;
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
            "PyMetric(name='{}', scope={:?}, rollup={:?}, count={})",
            self.inner.name(),
            self.inner.scope(),
            self.inner.rollup(),
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

    #[getter]
    pub fn scope(&self) -> String {
        format!("{:?}", self.inner.scope())
    }

    #[getter]
    pub fn rollup(&self) -> String {
        format!("{:?}", self.inner.rollup())
    }

    // --- value stats ---
    #[getter]
    pub fn value_last(&self) -> f32 {
        self.inner.last_value()
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

    // --- distribution summary (no big copies) ---
    #[getter]
    pub fn sequence_last(&self) -> Option<Vec<f32>> {
        self.inner.last_sequence().cloned()
    }

    #[getter]
    pub fn sequence_mean(&self) -> Option<f32> {
        self.inner.distribution_mean()
    }

    #[getter]
    pub fn sequence_stddev(&self) -> Option<f32> {
        self.inner.distribution_std_dev()
    }

    #[getter]
    pub fn sequence_min(&self) -> Option<f32> {
        self.inner.distribution_min()
    }

    #[getter]
    pub fn sequence_max(&self) -> Option<f32> {
        self.inner.distribution_max()
    }

    #[getter]
    pub fn sequence_variance(&self) -> Option<f32> {
        self.inner.distribution_variance()
    }

    #[getter]
    pub fn sequence_skewness(&self) -> Option<f32> {
        self.inner.distribution_skewness()
    }

    #[getter]
    pub fn sequence_kurtosis(&self) -> Option<f32> {
        self.inner.distribution_kurtosis()
    }

    /// ASCII sparkline of the last sequence; width default 40.
    #[pyo3(signature = (width=40))]
    pub fn spark(&self, width: usize) -> Option<String> {
        let seq = self.inner.last_sequence()?;
        Some(sparkline(seq, width))
    }

    /// Convert to a dict (nice for DataFrame construction / JSON dumps).
    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let d = PyDict::new(py);
        d.set_item("name", self.inner.name().to_string())?;
        d.set_item("scope", format!("{:?}", self.inner.scope()))?;
        d.set_item("rollup", format!("{:?}", self.inner.rollup()))?;

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

        d.set_item("seq_last", self.sequence_last())?;
        d.set_item("seq_mean", self.sequence_mean())?;
        d.set_item("seq_stddev", self.sequence_stddev())?;
        d.set_item("seq_min", self.sequence_min())?;
        d.set_item("seq_max", self.sequence_max())?;
        d.set_item("seq_var", self.sequence_variance())?;
        d.set_item("seq_skew", self.sequence_skewness())?;
        d.set_item("seq_kurtosis", self.sequence_kurtosis())?;

        Ok(d)
    }
}

fn sparkline(values: &[f32], width: usize) -> String {
    if values.is_empty() || width == 0 {
        return String::new();
    }
    let blocks = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let (mut mn, mut mx) = (f32::INFINITY, f32::NEG_INFINITY);
    for &v in values {
        if v < mn {
            mn = v
        }
        if v > mx {
            mx = v
        }
    }
    let span = (mx - mn).max(1e-12);
    let step = (values.len() as f32 / width as f32).max(1.0);
    let mut out = String::with_capacity(width);
    let mut idx = 0.0;
    for _ in 0..width {
        let i = f32::floor(idx) as usize;
        let v = values[i.min(values.len() - 1)];
        let level = (((v - mn) / span) * ((blocks.len() - 1) as f32)).round() as usize;
        out.push(blocks[level.min(blocks.len() - 1)]);
        idx += step;
    }
    out
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
