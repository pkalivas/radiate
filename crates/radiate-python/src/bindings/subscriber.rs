use crate::{IntoPyAnyObject, PyAnyObject, PyMetricSet, bindings::subscriber};
use numpy::PyArray1;
use pyo3::{IntoPyObjectExt, Py, PyAny, PyResult, Python, pyclass, pymethods};
use radiate::{
    Chromosome, EpochComplete, EventCtx, GeneticEngineBuilder, LimitTriggered, Objective,
    message::{
        CheckpointSaved, EngineStart, EngineStop, EpochStart, EventHandler, Improvement,
        LimitProgress, LogEvent,
    },
};
use std::fmt::Debug;

const EVENT_TYPES: &[&str] = &[
    crate::constants::components::START_EVENT,
    crate::constants::components::STOP_EVENT,
    crate::constants::components::EPOCH_START_EVENT,
    crate::constants::components::EPOCH_COMPLETE_EVENT,
    crate::constants::components::ENGINE_IMPROVEMENT_EVENT,
    crate::constants::components::LIMIT_TRIGGERED_EVENT,
    crate::constants::components::LOG_EVENT,
    crate::constants::components::CHECKPOINT_SAVED_EVENT,
    crate::constants::components::LIMIT_PROGRESS_EVENT,
];

#[pyclass(from_py_object)]
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

impl EventHandler<EngineStart> for PySubscriber {
    fn handle(&mut self, _: &EngineStart, _: &EventCtx) {
        Python::attach(|py| {
            let py_event = subscriber::PyEngineEvent::start();
            self.function
                .inner
                .call1(py, (py_event,))
                .expect("Failed to call subscriber function");
        })
    }
}

impl<T> EventHandler<EngineStop<T>> for PySubscriber
where
    T: IntoPyAnyObject + Clone,
{
    fn handle(&mut self, event: &EngineStop<T>, _: &EventCtx) {
        Python::attach(|py| {
            let py_event = subscriber::PyEngineEvent::stop(
                event.index,
                event
                    .best
                    .clone()
                    .into_py(py)
                    .expect("Failed to convert event."),
                subscriber::PyMetricSet::from(event.metrics.clone()),
                event.score.as_ref().to_vec(),
            );
            self.function
                .inner
                .call1(py, (py_event,))
                .expect("Failed to call subscriber function");
        })
    }
}

impl EventHandler<EpochStart> for PySubscriber {
    fn handle(&mut self, event: &EpochStart, _: &EventCtx) {
        Python::attach(|py| {
            let py_event = subscriber::PyEngineEvent::epoch_start(event.index);
            self.function
                .inner
                .call1(py, (py_event,))
                .expect("Failed to call subscriber function");
        })
    }
}

impl<T> EventHandler<EpochComplete<T>> for PySubscriber
where
    T: IntoPyAnyObject + Clone,
{
    fn handle(&mut self, event: &EpochComplete<T>, _: &EventCtx) {
        Python::attach(|py| {
            let py_event = subscriber::PyEngineEvent::epoch_complete(
                event.index,
                event
                    .best
                    .clone()
                    .into_py(py)
                    .expect("Failed to convert event."),
                subscriber::PyMetricSet::from(event.metrics.clone()),
                event.score.as_ref().to_vec(),
                event.objective.clone(),
            );
            self.function
                .inner
                .call1(py, (py_event,))
                .expect("Failed to call subscriber function");
        })
    }
}

impl<T> EventHandler<Improvement<T>> for PySubscriber
where
    T: IntoPyAnyObject + Clone,
{
    fn handle(&mut self, event: &Improvement<T>, _: &EventCtx) {
        Python::attach(|py| {
            let py_event = subscriber::PyEngineEvent::improvement(
                event.index,
                event
                    .best
                    .clone()
                    .into_py(py)
                    .expect("Failed to convert event."),
                event.score.as_ref().to_vec(),
            );
            self.function
                .inner
                .call1(py, (py_event,))
                .expect("Failed to call subscriber function");
        })
    }
}

impl EventHandler<LimitTriggered> for PySubscriber {
    fn handle(&mut self, event: &LimitTriggered, _: &EventCtx) {
        Python::attach(|py| {
            let py_event =
                subscriber::PyEngineEvent::limit_triggered(event.0, Some(format!("{:?}", event.1)));
            self.function
                .inner
                .call1(py, (py_event,))
                .expect("Failed to call subscriber function");
        })
    }
}

impl EventHandler<LogEvent> for PySubscriber {
    fn handle(&mut self, event: &LogEvent, _: &EventCtx) {
        Python::attach(|py| {
            let py_event = subscriber::PyEngineEvent::log_event(event.1.clone());
            self.function
                .inner
                .call1(py, (py_event,))
                .expect("Failed to call subscriber function");
        })
    }
}

impl EventHandler<CheckpointSaved> for PySubscriber {
    fn handle(&mut self, event: &CheckpointSaved, _: &EventCtx) {
        Python::attach(|py| {
            let py_event = PyEngineEvent::checkpoint_saved(event.index, event.path.clone());
            self.function
                .inner
                .call1(py, (py_event,))
                .expect("Failed to call subscriber function");
        })
    }
}

impl EventHandler<LimitProgress> for PySubscriber {
    fn handle(&mut self, event: &LimitProgress, _: &EventCtx) {
        Python::attach(|py| {
            let py_event = PyEngineEvent::limit_progress_event(event.clone());
            self.function
                .inner
                .call1(py, (py_event,))
                .expect("Failed to call subscriber function");
        })
    }
}

pub(crate) fn subscribe_python<C, T>(
    mut builder: GeneticEngineBuilder<C, T>,
    subscribers: Vec<PySubscriber>,
) -> GeneticEngineBuilder<C, T>
where
    C: Chromosome + PartialEq + Clone,
    T: IntoPyAnyObject + Send + Sync + Clone + 'static,
{
    use crate::constants::components;

    let equals_or_all = |name: &str, target: &str| name == target || name == components::ALL_EVENTS;

    for subscriber in subscribers {
        let event_type = subscriber
            .event_name()
            .map(|name| name.to_string())
            .unwrap_or_else(|| components::ALL_EVENTS.to_string());

        for &event_name in EVENT_TYPES {
            builder = if equals_or_all(&event_type, event_name) {
                match event_name {
                    components::START_EVENT => builder.subscribe::<EngineStart>(subscriber.clone()),
                    components::STOP_EVENT => {
                        builder.subscribe::<EngineStop<T>>(subscriber.clone())
                    }
                    components::EPOCH_START_EVENT => {
                        builder.subscribe::<EpochStart>(subscriber.clone())
                    }
                    components::EPOCH_COMPLETE_EVENT => {
                        builder.subscribe::<EpochComplete<T>>(subscriber.clone())
                    }
                    components::ENGINE_IMPROVEMENT_EVENT => {
                        builder.subscribe::<Improvement<T>>(subscriber.clone())
                    }
                    components::LIMIT_TRIGGERED_EVENT => {
                        builder.subscribe::<LimitTriggered>(subscriber.clone())
                    }
                    components::LOG_EVENT => builder.subscribe::<LogEvent>(subscriber.clone()),
                    components::CHECKPOINT_SAVED_EVENT => {
                        builder.subscribe::<CheckpointSaved>(subscriber.clone())
                    }
                    components::LIMIT_PROGRESS_EVENT => {
                        builder.subscribe::<LimitProgress>(subscriber.clone())
                    }
                    _ => builder,
                }
            } else {
                builder
            };
        }
    }

    builder
}

pub struct PyLimitEvent {
    pub kind: String,
}

#[pyclass]
pub struct PyEngineEvent {
    pub event_type: String,
    pub index: Option<usize>,
    pub best: Option<Py<PyAny>>,
    pub score: Option<Vec<f32>>,
    pub metrics: Option<PyMetricSet>,
    pub objective: Option<Vec<&'static str>>,
    pub description: Option<String>,
    pub limit_progress: Option<PyLimitEvent>,
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
            Some(scores) => PyArray1::from_vec(py, scores.clone()).into_py_any(py),
            None => Ok(py.None()),
        }
    }

    pub fn metrics(&self) -> Option<PyMetricSet> {
        self.metrics.as_ref().cloned()
    }

    pub fn objective(&self) -> Option<Vec<&'static str>> {
        self.objective.as_ref().cloned()
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn limit(&self) -> Option<String> {
        self.limit_progress.as_ref().map(|lim| lim.kind.clone())
    }
}

impl PyEngineEvent {
    pub fn start() -> PyEngineEvent {
        PyEngineEvent {
            event_type: crate::constants::components::START_EVENT.into(),
            index: None,
            best: None,
            score: None,
            metrics: None,
            objective: None,
            description: None,
            limit_progress: None,
        }
    }

    pub fn stop(
        generation: usize,
        best: PyAnyObject,
        metrics: PyMetricSet,
        score: Vec<f32>,
    ) -> PyEngineEvent {
        PyEngineEvent {
            event_type: crate::constants::components::STOP_EVENT.into(),
            index: Some(generation),
            best: Some(best.inner),
            score: Some(score),
            metrics: Some(metrics),
            objective: None,
            description: None,
            limit_progress: None,
        }
    }

    pub fn epoch_start(idx: usize) -> PyEngineEvent {
        PyEngineEvent {
            event_type: crate::constants::components::EPOCH_START_EVENT.into(),
            index: Some(idx),
            best: None,
            score: None,
            metrics: None,
            objective: None,
            description: None,
            limit_progress: None,
        }
    }

    pub fn epoch_complete(
        idx: usize,
        best: PyAnyObject,
        metrics: PyMetricSet,
        score: Vec<f32>,
        objective: Objective,
    ) -> PyEngineEvent {
        PyEngineEvent {
            event_type: crate::constants::components::EPOCH_COMPLETE_EVENT.into(),
            index: Some(idx),
            best: Some(best.inner),
            score: Some(score),
            metrics: Some(metrics),
            objective: Some(objective.into()),
            description: None,
            limit_progress: None,
        }
    }

    pub fn improvement(idx: usize, best: PyAnyObject, score: Vec<f32>) -> PyEngineEvent {
        PyEngineEvent {
            event_type: crate::constants::components::ENGINE_IMPROVEMENT_EVENT.into(),
            index: Some(idx),
            best: Some(best.inner),
            score: Some(score),
            metrics: None,
            objective: None,
            description: None,
            limit_progress: None,
        }
    }

    pub fn limit_triggered(idx: usize, description: Option<String>) -> PyEngineEvent {
        PyEngineEvent {
            event_type: crate::constants::components::LIMIT_TRIGGERED_EVENT.into(),
            index: Some(idx),
            best: None,
            score: None,
            metrics: None,
            objective: None,
            description,
            limit_progress: None,
        }
    }

    pub fn log_event(description: String) -> PyEngineEvent {
        PyEngineEvent {
            event_type: crate::constants::components::LOG_EVENT.into(),
            index: None,
            best: None,
            score: None,
            metrics: None,
            objective: None,
            description: Some(description),
            limit_progress: None,
        }
    }

    pub fn checkpoint_saved(idx: usize, path: String) -> PyEngineEvent {
        PyEngineEvent {
            event_type: crate::constants::components::CHECKPOINT_SAVED_EVENT.into(),
            index: Some(idx),
            best: None,
            score: None,
            metrics: None,
            objective: None,
            description: Some(path),
            limit_progress: None,
        }
    }

    pub fn limit_progress_event(lim: LimitProgress) -> PyEngineEvent {
        PyEngineEvent {
            event_type: crate::constants::components::LIMIT_PROGRESS_EVENT.into(),
            index: Some(lim.index()),
            best: None,
            score: None,
            metrics: None,
            objective: None,
            description: Some(lim.description()),
            limit_progress: Some(PyLimitEvent {
                kind: lim.kind().to_string(),
            }),
        }
    }
}
