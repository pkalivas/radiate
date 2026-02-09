use pyo3::{intern, prelude::*, pybacked::PyBackedStr, types::PyList};
use radiate::{DataType, Field};

use crate::Wrap;

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
                _ => return DataType::Null,
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

impl<'py> IntoPyObject<'py> for &Wrap<DataType> {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        use super::radiate;
        let rd = radiate(py).bind(py);

        match &self.0 {
            DataType::Int8 => {
                let class = rd.getattr(intern!(py, "Int8"))?;
                class.call0()
            }
            DataType::Int16 => {
                let class = rd.getattr(intern!(py, "Int16"))?;
                class.call0()
            }
            DataType::Int32 => {
                let class = rd.getattr(intern!(py, "Int32"))?;
                class.call0()
            }
            DataType::Int64 => {
                let class = rd.getattr(intern!(py, "Int64"))?;
                class.call0()
            }
            DataType::UInt8 => {
                let class = rd.getattr(intern!(py, "UInt8"))?;
                class.call0()
            }
            DataType::UInt16 => {
                let class = rd.getattr(intern!(py, "UInt16"))?;
                class.call0()
            }
            DataType::UInt32 => {
                let class = rd.getattr(intern!(py, "UInt32"))?;
                class.call0()
            }
            DataType::UInt64 => {
                let class = rd.getattr(intern!(py, "UInt64"))?;
                class.call0()
            }
            DataType::UInt128 => {
                let class = rd.getattr(intern!(py, "UInt128"))?;
                class.call0()
            }
            DataType::Int128 => {
                let class = rd.getattr(intern!(py, "Int128"))?;
                class.call0()
            }
            DataType::Float32 => {
                let class = rd.getattr(intern!(py, "Float32"))?;
                class.call0()
            }
            DataType::Float64 => {
                let class = rd.getattr(intern!(py, "Float64"))?;
                class.call0()
            }
            DataType::Boolean => {
                let class = rd.getattr(intern!(py, "Boolean"))?;
                class.call0()
            }
            DataType::String => {
                let class = rd.getattr(intern!(py, "String"))?;
                class.call0()
            }
            DataType::Char => {
                let class = rd.getattr(intern!(py, "Char"))?;
                class.call0()
            }
            DataType::List(inner) => {
                let class = rd.getattr(intern!(py, "List"))?;
                let inner = Wrap(*inner.clone());
                class.call1((&inner,))
            }
            DataType::Struct(fields) => {
                let field_class = rd.getattr(intern!(py, "Field"))?;
                let iter = fields.iter().map(|fld| {
                    let name = fld.name().as_str();
                    let dtype = Wrap(fld.dtype().clone());
                    field_class.call1((name, &dtype)).unwrap()
                });
                let fields = PyList::new(py, iter)?;
                let struct_class = rd.getattr(intern!(py, "Struct"))?;
                struct_class.call1((fields,))
            }
            DataType::Null => {
                let class = rd.getattr(intern!(py, "Null"))?;
                class.call0()
            }
        }
    }
}
impl<'a, 'py> FromPyObject<'a, 'py> for Wrap<Field> {
    type Error = PyErr;

    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> PyResult<Self> {
        let name = ob
            .getattr(intern!(ob.py(), "name"))?
            .str()?
            .extract::<PyBackedStr>()?;
        let dtype = ob
            .getattr(intern!(ob.py(), "dtype"))?
            .extract::<Wrap<DataType>>()?;
        Ok(Wrap(Field::new((&*name).into(), dtype.0)))
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for Wrap<DataType> {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let py = obj.py();
        let type_name = obj.get_type().qualname()?.to_string();

        let dtype = match &*type_name {
            "DataTypeClass" => {
                // just the class, not an object
                let name = obj
                    .getattr(intern!(py, "__name__"))?
                    .str()?
                    .extract::<PyBackedStr>()?;
                match &*name {
                    "Int8" => DataType::Int8,
                    "Int16" => DataType::Int16,
                    "Int32" => DataType::Int32,
                    "Int64" => DataType::Int64,
                    "UInt8" => DataType::UInt8,
                    "UInt16" => DataType::UInt16,
                    "UInt32" => DataType::UInt32,
                    "UInt64" => DataType::UInt64,
                    "UInt128" => DataType::UInt128,
                    "Int128" => DataType::Int128,
                    "Float32" => DataType::Float32,
                    "Float64" => DataType::Float64,
                    "Boolean" => DataType::Boolean,
                    "String" => DataType::String,
                    "Char" => DataType::Char,

                    _ => {
                        return Err(pyo3::exceptions::PyValueError::new_err(format!(
                            "Unsupported DataType class: {name}"
                        )));
                    }
                }
            }
            "Int8" => DataType::Int8,
            "Int16" => DataType::Int16,
            "Int32" => DataType::Int32,
            "Int64" => DataType::Int64,
            "Int128" => DataType::Int128,

            "UInt8" => DataType::UInt8,
            "UInt16" => DataType::UInt16,
            "UInt32" => DataType::UInt32,
            "UInt64" => DataType::UInt64,
            "UInt128" => DataType::UInt128,

            "Float32" => DataType::Float32,
            "Float64" => DataType::Float64,

            "Char" => DataType::Char,
            "String" => DataType::String,

            "Boolean" => DataType::Boolean,
            "List" => {
                let inner = obj.getattr(intern!(py, "inner")).unwrap();
                let inner = inner.extract::<Wrap<DataType>>()?;
                DataType::List(Box::new(inner.0))
            }

            "Struct" => {
                let fields = obj.getattr(intern!(py, "fields"))?;
                let fields = fields
                    .extract::<Vec<Wrap<Field>>>()?
                    .into_iter()
                    .map(|f| f.0)
                    .collect::<Vec<Field>>();
                DataType::Struct(fields)
            }

            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Unsupported DataType object of type: {type_name}"
                )));
            }
        };
        Ok(Wrap(dtype))
    }
}
