mod json;
mod wrap;
pub use wrap::*;

use pyo3::prelude::FromPyObjectOwned;
use pyo3::{Borrowed, IntoPyObject, PyAny, Python, types::PyAnyMethods};
use pyo3::{Py, PyResult};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

pub trait IntoPyAnyObject {
    fn into_py<'py>(self, py: Python<'py>) -> PyAnyObject;
}

impl IntoPyAnyObject for PyAnyObject {
    fn into_py<'py>(self, _: Python<'py>) -> PyAnyObject {
        self
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct PyAnyObject {
    pub inner: Py<PyAny>,
}

impl PyAnyObject {
    pub fn from_json<'py>(py: Python<'py>, json: &str) -> PyResult<Self> {
        let json_module = py.import("json")?;
        let loads = json_module.getattr("loads")?;
        let obj = loads.call1((json,))?;

        Ok(PyAnyObject { inner: obj.into() })
    }

    pub fn to_json<'py>(&self, py: Python<'py>) -> PyResult<String> {
        let bound = self.inner.bind_borrowed(py);
        let result = json::try_to_json(bound)?;
        Ok(result)
    }

    pub fn extract<'py, T: FromPyObjectOwned<'py>>(&self, py: Python<'py>) -> PyResult<T> {
        self.inner.as_any().extract::<T>(py).map_err(Into::into)
    }
}

impl Clone for PyAnyObject {
    fn clone(&self) -> Self {
        Python::attach(|py| Self {
            inner: self.inner.clone_ref(py),
        })
    }
}

impl Hash for PyAnyObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let h = Python::attach(|py| self.inner.bind(py).hash().expect("should be hashable"));
        state.write_isize(h)
    }
}

impl Display for PyAnyObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl From<Py<PyAny>> for PyAnyObject {
    fn from(p: Py<PyAny>) -> Self {
        Self { inner: p }
    }
}

impl<'a, 'py> IntoPyObject<'py> for &'a PyAnyObject {
    type Target = PyAny;
    type Output = Borrowed<'a, 'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.inner.bind_borrowed(py))
    }
}

impl Serialize for PyAnyObject {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let json_str = Python::attach(|py| self.to_json(py).map_err(serde::ser::Error::custom))?;
        serializer.serialize_str(&json_str)
    }
}

impl<'de> Deserialize<'de> for PyAnyObject {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json_str = String::deserialize(deserializer)?;
        Python::attach(|py| PyAnyObject::from_json(py, &json_str).map_err(serde::de::Error::custom))
    }
}

impl Default for PyAnyObject {
    fn default() -> Self {
        Python::attach(|py| PyAnyObject { inner: py.None() })
    }
}

unsafe impl Send for PyAnyObject {}
unsafe impl Sync for PyAnyObject {}
