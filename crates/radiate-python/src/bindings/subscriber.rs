use crate::{IntoPyAnyObject, PyAnyObject, PyMetricSet, bindings::subscriber, radiate};
use pyo3::{
    Py, PyAny, Python, intern, pyclass, pymethods,
    types::{PyAnyMethods, PyDict},
};
use radiate::{
    Chromosome, EpochComplete, EventContext, GeneticEngineBuilder, Handler, LimitTriggered,
    events::{CheckpointSaved, EngineStop, EpochStart, Improvement, LogEvent, LogLevel},
};
use std::fmt::Debug;

const EVENT_TYPES: &[&str] = &[
    crate::constants::event_types::START_EVENT,
    crate::constants::event_types::STOP_EVENT,
    crate::constants::event_types::EPOCH_START_EVENT,
    crate::constants::event_types::EPOCH_COMPLETE_EVENT,
    crate::constants::event_types::ENGINE_IMPROVEMENT_EVENT,
    crate::constants::event_types::LIMIT_TRIGGERED_EVENT,
    crate::constants::event_types::LOG_EVENT,
    crate::constants::event_types::CHECKPOINT_SAVED_EVENT,
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

impl<T> Handler<EngineStop<T>> for PySubscriber
where
    T: IntoPyAnyObject + Clone + Send + Sync + 'static,
{
    fn handle(&mut self, event: &EngineStop<T>, _: &EventContext<'_, Self>) {
        Python::attach(|py| {
            let rd = radiate(py).bind(py);
            let dict = PyDict::new(py);
            dict.set_item(intern!(py, "score"), event.score.as_ref().to_vec())
                .unwrap();
            dict.set_item(
                intern!(py, "metrics"),
                subscriber::PyMetricSet::from(event.metrics.clone()),
            )
            .unwrap();
            dict.set_item(
                intern!(py, "best"),
                event
                    .best
                    .clone()
                    .into_py(py)
                    .expect("Failed to convert event.")
                    .inner,
            )
            .unwrap();

            let py_event = rd
                .getattr(intern!(py, "EngineEvent"))
                .expect("Failed to get EngineEvent class")
                .call1((
                    crate::constants::event_types::STOP_EVENT,
                    Some(event.index),
                    dict.into_any().unbind(),
                ))
                .expect("Failed to create EngineEvent instance");

            self.function
                .inner
                .call1(py, (py_event,))
                .expect("Failed to call subscriber function");
        })
    }
}

impl Handler<EpochStart> for PySubscriber {
    fn handle(&mut self, event: &EpochStart, _: &EventContext<'_, Self>) {
        Python::attach(|py| {
            let rd = radiate(py).bind(py);
            let py_event = rd
                .getattr(intern!(py, "EngineEvent"))
                .expect("Failed to get EngineEvent class")
                .call1((
                    crate::constants::event_types::EPOCH_START_EVENT,
                    Some(event.0),
                    py.None(),
                ))
                .expect("Failed to create EngineEvent instance");

            self.function
                .inner
                .call1(py, (py_event,))
                .expect("Failed to call subscriber function");
        })
    }
}

impl<T> Handler<EpochComplete<T>> for PySubscriber
where
    T: IntoPyAnyObject + Clone + Send + Sync + 'static,
{
    fn handle(&mut self, event: &EpochComplete<T>, _: &EventContext<'_, Self>) {
        Python::attach(|py| {
            let rd = radiate(py).bind(py);
            let dict = PyDict::new(py);
            let objective: Vec<&'static str> = event.objective.clone().into();

            dict.set_item(intern!(py, "score"), event.score.as_ref().to_vec())
                .unwrap();
            dict.set_item(intern!(py, "objective"), objective).unwrap();
            dict.set_item(
                intern!(py, "metrics"),
                subscriber::PyMetricSet::from(event.metrics.clone()),
            )
            .unwrap();
            dict.set_item(
                intern!(py, "best"),
                event
                    .best
                    .clone()
                    .into_py(py)
                    .expect("Failed to convert event.")
                    .inner,
            )
            .unwrap();

            let py_event = rd
                .getattr(intern!(py, "EngineEvent"))
                .expect("Failed to get EngineEvent class")
                .call1((
                    crate::constants::event_types::EPOCH_COMPLETE_EVENT,
                    Some(event.index),
                    dict.into_any().unbind(),
                ))
                .expect("Failed to create EngineEvent instance");

            self.function
                .inner
                .call1(py, (py_event,))
                .expect("Failed to call subscriber function");
        })
    }
}

impl<T> Handler<Improvement<T>> for PySubscriber
where
    T: IntoPyAnyObject + Clone + Send + Sync + 'static,
{
    fn handle(&mut self, event: &Improvement<T>, _: &EventContext<'_, Self>) {
        Python::attach(|py| {
            let rd = radiate(py).bind(py);
            let dict = PyDict::new(py);
            dict.set_item(
                intern!(py, "score"),
                event.score.clone().as_slice().to_vec(),
            )
            .unwrap();

            let py_event = rd
                .getattr(intern!(py, "EngineEvent"))
                .expect("Failed to get EngineEvent class")
                .call1((
                    crate::constants::event_types::ENGINE_IMPROVEMENT_EVENT,
                    Some(event.index),
                    dict.into_any().unbind(),
                ))
                .expect("Failed to create EngineEvent instance");

            self.function
                .inner
                .call1(py, (py_event,))
                .expect("Failed to call subscriber function");
        })
    }
}

impl Handler<LimitTriggered> for PySubscriber {
    fn handle(&mut self, event: &LimitTriggered, _: &EventContext<'_, Self>) {
        Python::attach(|py| {
            let rd = radiate(py).bind(py);
            let py_dict = PyDict::new(py);
            py_dict
                .set_item(intern!(py, "limit"), format!("{:?}", event.1))
                .unwrap();

            let class = rd
                .getattr(intern!(py, "EngineEvent"))
                .expect("Failed to get EngineEvent class");

            let py_event = class
                .call1((
                    crate::constants::event_types::LIMIT_TRIGGERED_EVENT,
                    Some(event.0),
                    py_dict.into_any().unbind(),
                ))
                .expect("Failed to create EngineEvent instance");

            self.function
                .inner
                .call1(py, (py_event,))
                .expect("Failed to call subscriber function");
        })
    }
}

impl Handler<LogEvent> for PySubscriber {
    fn handle(&mut self, event: &LogEvent, _: &EventContext<'_, Self>) {
        Python::attach(|py| {
            let rd = radiate(py).bind(py);
            let py_dict = PyDict::new(py);
            py_dict
                .set_item(intern!(py, "log"), event.1.clone())
                .unwrap();
            py_dict
                .set_item(
                    intern!(py, "level"),
                    match event.0 {
                        LogLevel::Warn => "WARN",
                        LogLevel::Info => "INFO",
                    },
                )
                .unwrap();

            let class = rd
                .getattr(intern!(py, "EngineEvent"))
                .expect("Failed to get EngineEvent class");

            let py_event = class
                .call1((
                    crate::constants::event_types::LOG_EVENT,
                    None::<usize>,
                    py_dict.into_any().unbind(),
                ))
                .expect("Failed to create EngineEvent instance");

            self.function
                .inner
                .call1(py, (py_event,))
                .expect("Failed to call subscriber function");
        })
    }
}

impl Handler<CheckpointSaved> for PySubscriber {
    fn handle(&mut self, event: &CheckpointSaved, _: &EventContext<'_, Self>) {
        Python::attach(|py| {
            let rd = radiate(py).bind(py);
            let py_dict = PyDict::new(py);
            py_dict
                .set_item(intern!(py, "path"), event.path.clone())
                .unwrap();

            let class = rd
                .getattr(intern!(py, "EngineEvent"))
                .expect("Failed to get EngineEvent class");

            let py_event = class
                .call1((
                    crate::constants::event_types::CHECKPOINT_SAVED_EVENT,
                    Some(event.index),
                    py_dict.into_any().unbind(),
                ))
                .expect("Failed to create EngineEvent instance");

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
    use crate::constants::event_types;

    let equals_or_all =
        |name: &str, target: &str| name == target || name == event_types::ALL_EVENTS;

    for subscriber in subscribers {
        let event_type = subscriber
            .event_name()
            .map(|name| name.to_string())
            .unwrap_or_else(|| event_types::ALL_EVENTS.to_string());

        for &event_name in EVENT_TYPES {
            builder = if equals_or_all(&event_type, event_name) {
                match event_name {
                    event_types::STOP_EVENT => {
                        builder.subscribe::<EngineStop<T>>(subscriber.clone())
                    }
                    event_types::EPOCH_START_EVENT => {
                        builder.subscribe::<EpochStart>(subscriber.clone())
                    }
                    event_types::EPOCH_COMPLETE_EVENT => {
                        builder.subscribe::<EpochComplete<T>>(subscriber.clone())
                    }
                    event_types::ENGINE_IMPROVEMENT_EVENT => {
                        builder.subscribe::<Improvement<T>>(subscriber.clone())
                    }
                    event_types::LIMIT_TRIGGERED_EVENT => {
                        builder.subscribe::<LimitTriggered>(subscriber.clone())
                    }
                    event_types::LOG_EVENT => builder.subscribe::<LogEvent>(subscriber.clone()),
                    event_types::CHECKPOINT_SAVED_EVENT => {
                        builder.subscribe::<CheckpointSaved>(subscriber.clone())
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
