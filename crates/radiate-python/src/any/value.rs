use crate::Wrap;
use pyo3::{
    Borrowed, Bound, FromPyObject, IntoPyObject, PyAny, PyErr, PyResult, Python,
    exceptions::PyValueError,
};
use radiate::AnyValue;

impl<'py> FromPyObject<'_, 'py> for Wrap<AnyValue<'py>> {
    type Error = PyErr;

    fn extract(ob: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
        super::py_object_to_any_value(ob, true).map_err(|e| {
            PyValueError::new_err(format!(
                "{e}\n\nHint: Try setting `strict=False` to allow passing data with mixed types."
            ))
        })
        .map(Wrap)
    }
}

impl<'py> IntoPyObject<'py> for Wrap<AnyValue<'_>> {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        super::any_value_into_py_object(self.0, py)
    }
}

impl<'py> IntoPyObject<'py> for &Wrap<AnyValue<'_>> {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Wrap(&self.0).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for Wrap<&AnyValue<'_>> {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        super::any_value_into_py_object_ref(self.0, py)
    }
}
