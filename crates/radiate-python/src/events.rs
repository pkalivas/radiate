use crate::{ObjectValue, PyFunc, conversion::metric_set_to_py_dict};
use pyo3::{
    Python,
    types::{PyAnyMethods, PyDict},
};
use radiate::{EngineEvent, Event, EventHandler};

pub struct PyEventHandler {
    handler: PyFunc,
}

impl PyEventHandler {
    pub fn new(handler: PyFunc) -> Self {
        PyEventHandler { handler }
    }
}

impl<'py, 'a> EventHandler<EngineEvent<ObjectValue>> for PyEventHandler {
    fn handle(&mut self, event: Event<EngineEvent<ObjectValue>>) {
        Python::with_gil(|py| {
            py.allow_threads(|| {
                Python::with_gil(|inner| {
                    let py = inner;

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
                            dict.set_item("metrics", metric_set_to_py_dict(py, metrics).unwrap())
                                .unwrap();
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
                            dict.set_item("metrics", metric_set_to_py_dict(py, metrics).unwrap())
                                .unwrap();
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

                    self.handler
                        .func
                        .inner
                        .call1(py, (dict,))
                        .expect("Failed to call event handler");
                });
            })
        })
    }
}
