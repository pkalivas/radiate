use crate::{ObjectValue, PySubscriber, conversion::Wrap};
use pyo3::{
    Python,
    types::{PyAnyMethods, PyDict},
};
use radiate::{EngineEvent, Event, EventHandler};

pub struct PyEventHandler {
    handlers: Vec<PySubscriber>,
}

impl PyEventHandler {
    pub fn new(handlers: Vec<PySubscriber>) -> Self {
        PyEventHandler { handlers }
    }
}

impl<'py, 'a> EventHandler<EngineEvent<ObjectValue>> for PyEventHandler {
    fn handle(&mut self, event: Event<EngineEvent<ObjectValue>>) {
        let subscribers = self
            .handlers
            .iter()
            .filter(|handler| {
                handler
                    .event_name()
                    .map(|name| {
                        if matches!(event.data(), EngineEvent::Start) {
                            name == "on_start"
                        } else if matches!(event.data(), EngineEvent::Stop { .. }) {
                            name == "on_stop"
                        } else if matches!(event.data(), EngineEvent::EpochStart(_)) {
                            name == "on_epoch_start"
                        } else if matches!(event.data(), EngineEvent::EpochComplete { .. }) {
                            name == "on_epoch_complete"
                        } else if matches!(event.data(), EngineEvent::StepStart(_)) {
                            name == "on_step_start"
                        } else if matches!(event.data(), EngineEvent::StepComplete(_)) {
                            name == "on_step_complete"
                        } else if matches!(event.data(), EngineEvent::EngineImprovement { .. }) {
                            name == "on_engine_improvement"
                        } else {
                            false
                        }
                    })
                    .unwrap_or(true)
            })
            .collect::<Vec<_>>();

        if subscribers.is_empty() {
            return;
        }

        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            dict.set_item("id", *event.id()).unwrap();
            match event.data() {
                EngineEvent::Start => {
                    dict.set_item("type", "start").unwrap();
                }
                EngineEvent::Stop {
                    metrics,
                    best,
                    score,
                } => {
                    dict.set_item("type", "stop").unwrap();
                    dict.set_item("metrics", Wrap(metrics.clone())).unwrap();
                    dict.set_item("best", best.inner.bind_borrowed(py)).unwrap();
                    dict.set_item("score", score.as_f32()).unwrap();
                }
                EngineEvent::EpochStart(index) => {
                    dict.set_item("type", "epoch_start").unwrap();
                    dict.set_item("index", index).unwrap();
                }
                EngineEvent::EpochComplete {
                    index,
                    metrics,
                    best,
                    score,
                } => {
                    dict.set_item("type", "epoch_complete").unwrap();
                    dict.set_item("index", index).unwrap();
                    dict.set_item("metrics", Wrap(metrics.clone())).unwrap();
                    dict.set_item("best", best.inner.bind_borrowed(py)).unwrap();
                    dict.set_item("score", score.as_f32()).unwrap();
                }
                EngineEvent::StepStart(step) => {
                    dict.set_item("type", "step_start").unwrap();
                    dict.set_item("step", step).unwrap();
                }
                EngineEvent::StepComplete(step) => {
                    dict.set_item("type", "step_complete").unwrap();
                    dict.set_item("step", step).unwrap();
                }
                EngineEvent::EngineImprovement { index, best, score } => {
                    dict.set_item("type", "engine_improvement").unwrap();
                    dict.set_item("index", index).unwrap();
                    dict.set_item("best", best.inner.bind_borrowed(py)).unwrap();
                    dict.set_item("score", score.as_f32()).unwrap();
                }
            }

            let unbound_event = dict.unbind();
            for handler in subscribers {
                let cloned_event = unbound_event.clone_ref(py);
                handler
                    .function()
                    .call1(py, (cloned_event,))
                    .expect("Failed to call event handler");
            }
        });
    }
}
