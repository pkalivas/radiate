use crate::{PyAnyObject, PyMetricSet};
use numpy::PyArray1;
use pyo3::{IntoPyObjectExt, Py, PyAny, PyResult, Python, pyclass, pymethods};
use std::fmt::Debug;

#[pyclass]
#[derive(Clone)]
pub struct PySubscriber {
    event_name: Option<String>,
    function: PyAnyObject,
}

#[pymethods]
impl PySubscriber {
    #[new]
    #[pyo3(signature = (function, event_name=None))]
    pub fn new(function: Py<PyAny>, event_name: Option<String>) -> Self {
        Self {
            event_name,
            function: PyAnyObject { inner: function },
        }
    }

    pub fn event_name(&self) -> Option<&str> {
        self.event_name.as_deref()
    }

    pub fn function(&self) -> &Py<PyAny> {
        &self.function.inner
    }
}

impl Debug for PySubscriber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PySubscriber")
            .field("event_name", &self.event_name)
            .finish()
    }
}

#[pyclass]
pub struct PyEngineEvent {
    pub event_type: String,
    pub index: Option<usize>,
    pub best: Option<Py<PyAny>>,
    pub score: Option<Vec<f32>>,
    pub metrics: Option<PyMetricSet>,
}

#[pymethods]
impl PyEngineEvent {
    fn __repr__(&self) -> String {
        format!(
            "PyEngineEvent(type={}, index={:?}, score={:?})",
            self.event_type, self.index, self.score
        )
    }

    pub fn event_type(&self) -> &str {
        &self.event_type
    }

    pub fn index(&self) -> Option<usize> {
        self.index
    }

    pub fn best(&self) -> Option<&Py<PyAny>> {
        self.best.as_ref()
    }

    pub fn score<'py>(&self, py: Python<'py>) -> PyResult<Py<PyAny>> {
        match &self.score {
            Some(scores) => {
                if scores.len() == 1 {
                    scores[0].into_py_any(py)
                } else {
                    PyArray1::from_vec(py, scores.clone()).into_py_any(py)
                }
            }
            None => Ok(py.None()),
        }
    }

    pub fn metrics(&self) -> Option<PyMetricSet> {
        self.metrics.as_ref().cloned()
    }
}

impl PyEngineEvent {
    pub fn start() -> PyEngineEvent {
        PyEngineEvent {
            event_type: crate::names::START_EVENT.into(),
            index: None,
            best: None,
            score: None,
            metrics: None,
        }
    }

    pub fn stop(best: PyAnyObject, metrics: PyMetricSet, score: Vec<f32>) -> PyEngineEvent {
        PyEngineEvent {
            event_type: crate::names::STOP_EVENT.into(),
            index: None,
            best: Some(best.inner),
            score: Some(score),
            metrics: Some(metrics),
        }
    }

    pub fn epoch_start(idx: usize) -> PyEngineEvent {
        PyEngineEvent {
            event_type: crate::names::EPOCH_START_EVENT.into(),
            index: Some(idx),
            best: None,
            score: None,
            metrics: None,
        }
    }

    pub fn epoch_complete(
        idx: usize,
        best: PyAnyObject,
        metrics: PyMetricSet,
        score: Vec<f32>,
    ) -> PyEngineEvent {
        PyEngineEvent {
            event_type: crate::names::EPOCH_COMPLETE_EVENT.into(),
            index: Some(idx),
            best: Some(best.inner),
            score: Some(score),
            metrics: Some(metrics),
        }
    }

    pub fn improvement(idx: usize, best: PyAnyObject, score: Vec<f32>) -> PyEngineEvent {
        PyEngineEvent {
            event_type: crate::names::ENGINE_IMPROVEMENT_EVENT.into(),
            index: Some(idx),
            best: Some(best.inner),
            score: Some(score),
            metrics: None,
        }
    }
}
