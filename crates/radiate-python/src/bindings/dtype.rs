use pyo3::prelude::*;
use radiate::DataType;

#[pyfunction]
pub fn _get_dtype_max<'py>(py: Python<'py>, dt: String) -> PyResult<Bound<'py, PyAny>> {
    let dt = DataType::from(dt);
    let max = dt.max();

    crate::any::any_value_into_py_object(
        max.ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!(
                "Data type {dt} does not have a defined maximum value."
            ))
        })?
        .into_value(),
        py,
    )
}

#[pyfunction]
pub fn _get_dtype_min<'py>(py: Python<'py>, dt: String) -> PyResult<Bound<'py, PyAny>> {
    let dt = DataType::from(dt);
    let min = dt.min();

    crate::any::any_value_into_py_object(
        min.ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!(
                "Data type {dt} does not have a defined minimum value."
            ))
        })?
        .into_value(),
        py,
    )
}
