use crate::prelude::*;
use crate::{PySubscriber, object::Wrap};
use pyo3::intern;
use pyo3::{Python, types::PyDict};
use radiate::{EngineEvent, Event, EventHandler};

const ON_START: &'static str = "on_start";
const ON_STOP: &'static str = "on_stop";
const ON_EPOCH_START: &'static str = "on_epoch_start";
const ON_EPOCH_COMPLETE: &'static str = "on_epoch_complete";
const ON_STEP_START: &'static str = "on_step_start";
const ON_STEP_COMPLETE: &'static str = "on_step_complete";
const ON_ENGINE_IMPROVEMENT: &'static str = "on_engine_improvement";

pub struct PyEventHandler {
    handlers: Vec<PySubscriber>,
}

impl PyEventHandler {
    pub fn new(handlers: Vec<PySubscriber>) -> Self {
        PyEventHandler { handlers }
    }

    fn get_valid_handlers(
        &self,
        event: &EngineEvent<impl IntoPyObjectValue>,
    ) -> Vec<&PySubscriber> {
        self.handlers
            .iter()
            .filter(|handler| {
                handler
                    .event_name()
                    .map(|name| {
                        if matches!(event, EngineEvent::Start) {
                            name == ON_START
                        } else if matches!(event, EngineEvent::Stop { .. }) {
                            name == ON_STOP
                        } else if matches!(event, EngineEvent::EpochStart(_)) {
                            name == ON_EPOCH_START
                        } else if matches!(event, EngineEvent::EpochComplete { .. }) {
                            name == ON_EPOCH_COMPLETE
                        } else if matches!(event, EngineEvent::StepStart(_)) {
                            name == ON_STEP_START
                        } else if matches!(event, EngineEvent::StepComplete(_)) {
                            name == ON_STEP_COMPLETE
                        } else if matches!(event, EngineEvent::EngineImprovement { .. }) {
                            name == ON_ENGINE_IMPROVEMENT
                        } else {
                            false
                        }
                    })
                    .unwrap_or(true)
            })
            .collect()
    }

    fn event_to_py_dict<T>(&self, py: Python, event: &Event<EngineEvent<T>>) -> Py<PyDict>
    where
        T: IntoPyObjectValue + Clone,
    {
        let dict = PyDict::new(py);
        dict.set_item(intern!(py, "id"), *event.id()).unwrap();

        match event.data() {
            EngineEvent::Start => {
                dict.set_item(intern!(py, "type"), "start").unwrap();
            }
            EngineEvent::Stop {
                metrics,
                best,
                score,
            } => {
                let best = best.clone().into_py(py);
                dict.set_item(intern!(py, "type"), "stop").unwrap();
                dict.set_item(intern!(py, "metrics"), Wrap(metrics.clone()))
                    .unwrap();
                dict.set_item(intern!(py, "best"), best.inner).unwrap();
                dict.set_item(intern!(py, "score"), score.as_f32()).unwrap();
            }
            EngineEvent::EpochStart(index) => {
                dict.set_item(intern!(py, "type"), "epoch_start").unwrap();
                dict.set_item(intern!(py, "index"), index).unwrap();
            }
            EngineEvent::EpochComplete {
                index,
                metrics,
                best,
                score,
            } => {
                let best = best.clone().into_py(py);
                dict.set_item(intern!(py, "type"), "epoch_complete")
                    .unwrap();
                dict.set_item(intern!(py, "index"), index).unwrap();
                dict.set_item(intern!(py, "metrics"), Wrap(metrics.clone()))
                    .unwrap();
                dict.set_item(intern!(py, "best"), best.inner).unwrap();
                dict.set_item(intern!(py, "score"), score.as_f32()).unwrap();
            }
            EngineEvent::StepStart(step) => {
                dict.set_item(intern!(py, "type"), "step_start").unwrap();
                dict.set_item(intern!(py, "step"), step).unwrap();
            }
            EngineEvent::StepComplete(step) => {
                dict.set_item(intern!(py, "type"), "step_complete").unwrap();
                dict.set_item(intern!(py, "step"), step).unwrap();
            }
            EngineEvent::EngineImprovement { index, best, score } => {
                let best = best.clone().into_py(py);
                dict.set_item(intern!(py, "type"), "engine_improvement")
                    .unwrap();
                dict.set_item(intern!(py, "index"), index).unwrap();
                dict.set_item(intern!(py, "best"), best.inner).unwrap();
                dict.set_item(intern!(py, "score"), score.as_f32()).unwrap();
            }
        }

        dict.unbind()
    }
}

impl<T> EventHandler<EngineEvent<T>> for PyEventHandler
where
    T: IntoPyObjectValue + Clone,
{
    fn handle(&mut self, event: Event<EngineEvent<T>>) {
        let subscribers = self.get_valid_handlers(event.data());

        if subscribers.is_empty() {
            return;
        }

        Python::with_gil(|py| {
            let event_dict = self.event_to_py_dict(py, &event);

            for handler in subscribers {
                let cloned_event = event_dict.clone_ref(py);
                handler
                    .function()
                    .call1(py, (cloned_event,))
                    .expect("Failed to call event handler");
            }
        });
    }
}
