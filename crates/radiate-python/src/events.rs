use crate::PySubscriber;
use crate::{PyEngineEvent, PyMetricSet, prelude::*};
use pyo3::Python;
use radiate::{EngineEvent, EngineEventInner, EventHandler};

pub struct PyEventHandler {
    handlers: Vec<PySubscriber>,
}

impl PyEventHandler {
    pub fn new(handlers: Vec<PySubscriber>) -> Self {
        PyEventHandler { handlers }
    }

    fn get_valid_handlers(&self, event: &EngineEvent<impl IntoPyAnyObject>) -> Vec<&PySubscriber> {
        self.handlers
            .iter()
            .filter(|handler| {
                handler
                    .event_name()
                    .map(|name| {
                        if name == crate::names::ALL_EVENTS {
                            true
                        } else if event.is_start() {
                            name == crate::names::START_EVENT
                        } else if event.is_stop() {
                            name == crate::names::STOP_EVENT
                        } else if event.is_epoch_start() {
                            name == crate::names::EPOCH_START_EVENT
                        } else if event.is_epoch_complete() {
                            name == crate::names::EPOCH_COMPLETE_EVENT
                        } else if event.is_improvement() {
                            name == crate::names::ENGINE_IMPROVEMENT_EVENT
                        } else {
                            false
                        }
                    })
                    .unwrap_or(true)
            })
            .collect()
    }

    fn event_to_py<T>(&self, py: Python<'_>, event: &EngineEvent<T>) -> PyEngineEvent
    where
        T: IntoPyAnyObject + Clone,
    {
        match event.inner() {
            EngineEventInner::Start => PyEngineEvent::start(),
            EngineEventInner::Stop(index, best, metrics, score) => {
                let best = best.clone().into_py(py);
                let metrics = PyMetricSet::from(metrics.clone());
                PyEngineEvent::stop(*index, best, metrics, score.as_ref().to_vec())
            }
            EngineEventInner::EpochStart(index) => PyEngineEvent::epoch_start(*index),
            EngineEventInner::EpochComplete(index, best, metrics, score, objective) => {
                let best = best.clone().into_py(py);
                let metrics = PyMetricSet::from(metrics.clone());
                PyEngineEvent::epoch_complete(
                    *index,
                    best,
                    metrics,
                    score.as_ref().to_vec(),
                    objective.clone(),
                )
            }
            EngineEventInner::Improvement(index, best, score) => {
                let best = best.clone().into_py(py);
                PyEngineEvent::improvement(*index, best, score.as_ref().to_vec())
            }
        }
    }
}

impl<T> EventHandler<T> for PyEventHandler
where
    T: IntoPyAnyObject + Clone,
{
    fn handle(&mut self, event: EngineEvent<T>) {
        let subscribers = self.get_valid_handlers(&event);

        if subscribers.is_empty() {
            return;
        }

        Python::attach(|py| {
            let py_event = self.event_to_py(py, &event).into_py_any(py).unwrap();

            for handler in subscribers {
                let cloned_event = py_event.clone_ref(py);
                handler
                    .function()
                    .call1(py, (cloned_event,))
                    .expect("Failed to call event handler");
            }
        });
    }
}
