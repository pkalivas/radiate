use crate::PySubscriber;
use crate::{PyEngineEvent, PyMetricSet, prelude::*};
use pyo3::Python;
use radiate::{EngineEvent, EventContext, EventHandler};

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
                        if name == crate::constants::components::ALL_EVENTS {
                            true
                        } else if event.is_start() {
                            name == crate::constants::components::START_EVENT
                        } else if event.is_stop() {
                            name == crate::constants::components::STOP_EVENT
                        } else if event.is_epoch_start() {
                            name == crate::constants::components::EPOCH_START_EVENT
                        } else if event.is_epoch_complete() {
                            name == crate::constants::components::EPOCH_COMPLETE_EVENT
                        } else if event.is_improvement() {
                            name == crate::constants::components::ENGINE_IMPROVEMENT_EVENT
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
        match event {
            EngineEvent::Started(_) => PyEngineEvent::start(),
            EngineEvent::Stopped(s) => {
                let best = s
                    .best
                    .clone()
                    .into_py(py)
                    .expect("Failed to convert event.");
                let metrics = PyMetricSet::from(s.metrics.clone());
                PyEngineEvent::stop(s.index, best, metrics, s.score.as_ref().to_vec())
            }
            EngineEvent::EpochStarted(s) => PyEngineEvent::epoch_start(s.index),
            EngineEvent::EpochCompleted(s) => {
                let best = s
                    .best
                    .clone()
                    .into_py(py)
                    .expect("Failed to convert event.");
                let metrics = PyMetricSet::from(s.metrics.clone());
                PyEngineEvent::epoch_complete(
                    s.index,
                    best,
                    metrics,
                    s.score.as_ref().to_vec(),
                    s.objective.clone(),
                )
            }
            EngineEvent::Improved(s) => {
                let best = s
                    .best
                    .clone()
                    .into_py(py)
                    .expect("Failed to convert event.");
                PyEngineEvent::improvement(s.index, best, s.score.as_ref().to_vec())
            }
        }
    }
}

impl<T> EventHandler<EngineEvent<T>> for PyEventHandler
where
    T: IntoPyAnyObject + Clone,
{
    fn handle(&mut self, event: EngineEvent<T>, _ctx: &EventContext) {
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
