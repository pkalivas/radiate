use crate::prelude::*;
use crate::{PySubscriber, conversion::Wrap};
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
}

impl<T> EventHandler<EngineEvent<T>> for PyEventHandler
where
    T: IntoPyObjectValue + Clone,
{
    fn handle(&mut self, event: Event<EngineEvent<T>>) {
        let subscribers = self
            .handlers
            .iter()
            .filter(|handler| {
                handler
                    .event_name()
                    .map(|name| {
                        if matches!(event.data(), EngineEvent::Start) {
                            name == ON_START
                        } else if matches!(event.data(), EngineEvent::Stop { .. }) {
                            name == ON_STOP
                        } else if matches!(event.data(), EngineEvent::EpochStart(_)) {
                            name == ON_EPOCH_START
                        } else if matches!(event.data(), EngineEvent::EpochComplete { .. }) {
                            name == ON_EPOCH_COMPLETE
                        } else if matches!(event.data(), EngineEvent::StepStart(_)) {
                            name == ON_STEP_START
                        } else if matches!(event.data(), EngineEvent::StepComplete(_)) {
                            name == ON_STEP_COMPLETE
                        } else if matches!(event.data(), EngineEvent::EngineImprovement { .. }) {
                            name == ON_ENGINE_IMPROVEMENT
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
                    let best = best.clone().into_py(py);

                    dict.set_item("type", "stop").unwrap();
                    dict.set_item("metrics", Wrap(metrics.clone())).unwrap();
                    dict.set_item("best", best.inner).unwrap();
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
                    let best = best.clone().into_py(py);
                    dict.set_item("type", "epoch_complete").unwrap();
                    dict.set_item("index", index).unwrap();
                    dict.set_item("metrics", Wrap(metrics.clone())).unwrap();
                    dict.set_item("best", best.inner).unwrap();
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
                    let best = best.clone().into_py(py);
                    dict.set_item("type", "engine_improvement").unwrap();
                    dict.set_item("index", index).unwrap();
                    dict.set_item("best", best.inner).unwrap();
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
