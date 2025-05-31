use pyo3::{PyObject, Python, pyclass, pymethods, types::PyAnyMethods};

use crate::ObjectValue;

#[pyclass]
#[derive(Clone)]
pub struct PyEventHandler {
    handler: ObjectValue,
}

#[pymethods]
impl PyEventHandler {
    #[new]
    pub fn new(handler: ObjectValue) -> Self {
        Self { handler }
    }

    // pub fn handle_event<'py>(&self, py: Python<'py>, event: ObjectValue) -> PyObject {
    //     Python::with_gil(|py| {
    //         self.handler
    //             .inner
    //             .call_method1(py, "handle", (event.inner.bind_borrowed(py),))
    //             .expect("Event handler should not fail")
    //             .into()
    //     })
    // }
}
