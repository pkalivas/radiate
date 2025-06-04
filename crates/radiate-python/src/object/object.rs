use pyo3::{Borrowed, IntoPyObject, PyAny, PyObject, Python, types::PyAnyMethods};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Debug)]
#[repr(transparent)]
pub struct ObjectValue {
    pub inner: PyObject,
}

impl Clone for ObjectValue {
    fn clone(&self) -> Self {
        Python::with_gil(|py| Self {
            inner: self.inner.clone_ref(py),
        })
    }
}

impl Hash for ObjectValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let h = Python::with_gil(|py| self.inner.bind(py).hash().expect("should be hashable"));
        state.write_isize(h)
    }
}

impl Display for ObjectValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl From<PyObject> for ObjectValue {
    fn from(p: PyObject) -> Self {
        Self { inner: p }
    }
}

impl<'a, 'py> IntoPyObject<'py> for &'a ObjectValue {
    type Target = PyAny;
    type Output = Borrowed<'a, 'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.inner.bind_borrowed(py))
    }
}

impl Default for ObjectValue {
    fn default() -> Self {
        Python::with_gil(|py| ObjectValue { inner: py.None() })
    }
}

unsafe impl Send for ObjectValue {}
unsafe impl Sync for ObjectValue {}
