use pyo3::prelude::FromPyObjectOwned;
use pyo3::types::{PyFloat, PyInt};
use pyo3::{Borrowed, IntoPyObject, PyAny, Python, types::PyAnyMethods};
use pyo3::{Py, PyResult};
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
    pub fn extract<'py, T: FromPyObjectOwned<'py>>(&self, py: Python<'py>) -> PyResult<T> {
        self.inner.as_any().extract::<T>(py).map_err(Into::into)
    }

    pub fn get_f32(&self) -> Option<f32> {
        Python::attach(|py| {
            let bound = self.inner.bind_borrowed(py);
            if bound.is_instance_of::<PyFloat>() {
                bound.extract::<f64>().ok().map(|v| v as f32)
            } else if bound.is_instance_of::<PyInt>() {
                bound.extract::<i64>().ok().map(|v| v as f32)
            } else {
                None
            }
        })
    }

    pub fn get_f64(&self) -> Option<f64> {
        Python::attach(|py| {
            let bound = self.inner.bind_borrowed(py);
            if bound.is_instance_of::<PyFloat>() {
                bound.extract::<f64>().ok()
            } else if bound.is_instance_of::<PyInt>() {
                bound.extract::<i64>().ok().map(|v| v as f64)
            } else {
                None
            }
        })
    }

    pub fn get_i32(&self) -> Option<i32> {
        Python::attach(|py| {
            let bound = self.inner.bind_borrowed(py);
            if bound.is_instance_of::<PyInt>() {
                bound.extract::<i64>().ok().map(|v| v as i32)
            } else if bound.is_instance_of::<PyFloat>() {
                bound.extract::<f64>().ok().map(|v| v as i32)
            } else {
                None
            }
        })
    }

    pub fn get_usize(&self) -> Option<usize> {
        Python::attach(|py| {
            let bound = self.inner.bind_borrowed(py);
            if bound.is_instance_of::<PyInt>() {
                bound.extract::<i64>().ok().map(|v| v as usize)
            } else if bound.is_instance_of::<PyFloat>() {
                bound.extract::<f64>().ok().map(|v| v as usize)
            } else {
                None
            }
        })
    }

    pub fn get_string(&self) -> Option<String> {
        Python::attach(|py| {
            let bound = self.inner.bind_borrowed(py);
            bound.extract::<String>().ok()
        })
    }

    pub fn get_bool(&self) -> Option<bool> {
        Python::attach(|py| {
            let bound = self.inner.bind_borrowed(py);
            bound.extract::<bool>().ok()
        })
    }

    pub fn get_vec_f32(&self) -> Option<Vec<f32>> {
        Python::attach(|py| {
            let bound = self.inner.bind_borrowed(py);
            bound.extract::<Vec<f32>>().ok()
        })
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

impl Default for PyAnyObject {
    fn default() -> Self {
        Python::attach(|py| PyAnyObject { inner: py.None() })
    }
}

unsafe impl Send for PyAnyObject {}
unsafe impl Sync for PyAnyObject {}
