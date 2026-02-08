use pyo3::prelude::*;
use radiate::DataType;

pub fn dtype_from_str(value: &str) -> DataType {
    // check to see if the value is a numpy dtype string like "numpy.float32"
    let value = value.trim().to_lowercase();
    if let Some(stripped) = value.strip_prefix('<') {
        if stripped.contains("numpy") {
            let dtype_str = stripped.trim_start_matches("numpy").split('.');
            let last_parsed = dtype_str
                .clone()
                .last()
                .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()));

            match last_parsed {
                Some("float32") => return DataType::Float32,
                Some("float64") => return DataType::Float64,

                Some("int8") => return DataType::Int8,
                Some("int16") => return DataType::Int16,
                Some("int32") => return DataType::Int32,
                Some("int64") => return DataType::Int64,

                Some("uint8") => return DataType::UInt8,
                Some("uint16") => return DataType::UInt16,
                Some("uint32") => return DataType::UInt32,
                Some("uint64") => return DataType::UInt64,

                Some("bool") => return DataType::Boolean,
                _ => return DataType::Unknown, // If it looks like a numpy dtype but we don't recognize it, return Unknown
            }
        }
    }

    DataType::from(value.to_string())
}

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
