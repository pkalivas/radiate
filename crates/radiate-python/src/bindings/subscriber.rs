use crate::ObjectValue;
use pyo3::{Py, PyAny, pyclass, pymethods};

#[pyclass]
#[derive(Clone)]
pub struct PySubscriber {
    event_name: Option<String>,
    function: ObjectValue,
}

#[pymethods]
impl PySubscriber {
    #[new]
    #[pyo3(signature = (function, event_name=None))]
    pub fn new(function: Py<PyAny>, event_name: Option<String>) -> Self {
        Self {
            event_name,
            function: ObjectValue { inner: function },
        }
    }

    pub fn event_name(&self) -> Option<&str> {
        self.event_name.as_deref()
    }

    pub fn function(&self) -> &Py<PyAny> {
        &self.function.inner
    }
}
