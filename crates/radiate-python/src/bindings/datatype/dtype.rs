use super::{Field, Scalar};
use pyo3::{intern, prelude::*, pybacked::PyBackedStr, types::PyList};
use radiate_utils::{Float, Integer};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::Wrap;

pub mod dtype_names {
    pub const NULL: &str = "null";
    pub const BOOLEAN: &str = "boolean";
    pub const UINT8: &str = "uint8";
    pub const UINT16: &str = "uint16";
    pub const UINT32: &str = "uint32";
    pub const UINT64: &str = "uint64";
    pub const UINT128: &str = "uint128";
    pub const INT8: &str = "int8";
    pub const INT16: &str = "int16";
    pub const INT32: &str = "int32";
    pub const INT64: &str = "int64";
    pub const INT128: &str = "int128";
    pub const FLOAT32: &str = "float32";
    pub const FLOAT64: &str = "float64";
    pub const USIZE: &str = "usize";
    pub const BINARY: &str = "binary";
    pub const CHAR: &str = "char";
    pub const STRING: &str = "string";
    pub const DATE: &str = "date";
    pub const VEC: &str = "vec";
    pub const STRUCT: &str = "struct";
    pub const OP32: &str = "op32";
    pub const NODE: &str = "node";
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DataType {
    #[default]
    Null,

    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,

    Int8,
    Int16,
    Int32,
    Int64,
    Int128,

    Float32,
    Float64,

    Boolean,

    Char,
    String,

    List(Box<DataType>),
    Struct(Vec<Field>),

    Op32,

    Node(Box<DataType>),
}

impl DataType {
    pub fn is_nested(&self) -> bool {
        use DataType as D;

        matches!(self, D::List(_) | D::Struct(_))
    }

    pub fn is_numeric(&self) -> bool {
        use DataType as D;
        matches!(
            self,
            D::Int8
                | D::Int16
                | D::Int32
                | D::Int64
                | D::Int128
                | D::UInt8
                | D::UInt16
                | D::UInt32
                | D::UInt64
                | D::Float32
                | D::Float64
        )
    }

    pub fn is_primitive(&self) -> bool {
        use DataType as D;
        matches!(
            self,
            D::Null
                | D::Boolean
                | D::Int8
                | D::Int16
                | D::Int32
                | D::Int64
                | D::Int128
                | D::UInt8
                | D::UInt16
                | D::UInt32
                | D::UInt64
                | D::Float32
                | D::Float64
        )
    }

    pub fn max(&self) -> Option<Scalar> {
        use DataType as D;
        match self {
            D::Int8 => Some(Scalar::from(<i8 as Integer>::MAX)),
            D::Int16 => Some(Scalar::from(<i16 as Integer>::MAX)),
            D::Int32 => Some(Scalar::from(<i32 as Integer>::MAX)),
            D::Int64 => Some(Scalar::from(<i64 as Integer>::MAX)),
            D::Int128 => Some(Scalar::from(<i128 as Integer>::MAX)),
            D::UInt8 => Some(Scalar::from(<u8 as Integer>::MAX)),
            D::UInt16 => Some(Scalar::from(<u16 as Integer>::MAX)),
            D::UInt32 => Some(Scalar::from(<u32 as Integer>::MAX)),
            D::UInt64 => Some(Scalar::from(<u64 as Integer>::MAX)),
            D::UInt128 => Some(Scalar::from(<u128 as Integer>::MAX)),
            D::Float32 => Some(Scalar::from(<f32 as Float>::MAX)),
            D::Float64 => Some(Scalar::from(<f64 as Float>::MAX)),
            _ => None,
        }
    }

    pub fn min(&self) -> Option<Scalar> {
        use DataType as D;
        match self {
            D::Int8 => Some(Scalar::from(<i8 as Integer>::MIN)),
            D::Int16 => Some(Scalar::from(<i16 as Integer>::MIN)),
            D::Int32 => Some(Scalar::from(<i32 as Integer>::MIN)),
            D::Int64 => Some(Scalar::from(<i64 as Integer>::MIN)),
            D::Int128 => Some(Scalar::from(<i128 as Integer>::MIN)),
            D::UInt8 => Some(Scalar::from(<u8 as Integer>::MIN)),
            D::UInt16 => Some(Scalar::from(<u16 as Integer>::MIN)),
            D::UInt32 => Some(Scalar::from(<u32 as Integer>::MIN)),
            D::UInt64 => Some(Scalar::from(<u64 as Integer>::MIN)),
            D::UInt128 => Some(Scalar::from(<u128 as Integer>::MIN)),
            D::Float32 => Some(Scalar::from(<f32 as Float>::MIN)),
            D::Float64 => Some(Scalar::from(<f64 as Float>::MIN)),
            _ => None,
        }
    }

    pub fn primitive_bounds(&self) -> Option<(Scalar, Scalar)> {
        match (self.min(), self.max()) {
            (Some(min), Some(max)) => Some((min, max)),
            _ => None,
        }
    }
}

impl From<String> for DataType {
    fn from(value: String) -> Self {
        match value.trim().to_lowercase().as_str() {
            dtype_names::NULL => DataType::Null,

            dtype_names::UINT8 => DataType::UInt8,
            dtype_names::UINT16 => DataType::UInt16,
            dtype_names::UINT32 => DataType::UInt32,
            dtype_names::UINT64 => DataType::UInt64,
            dtype_names::UINT128 => DataType::UInt128,

            dtype_names::INT8 => DataType::Int8,
            dtype_names::INT16 => DataType::Int16,
            dtype_names::INT32 => DataType::Int32,
            dtype_names::INT64 => DataType::Int64,
            dtype_names::INT128 => DataType::Int128,

            dtype_names::FLOAT32 => DataType::Float32,
            dtype_names::FLOAT64 => DataType::Float64,

            dtype_names::BOOLEAN => DataType::Boolean,

            dtype_names::CHAR => DataType::Char,
            dtype_names::STRING => DataType::String,

            dtype_names::VEC => DataType::List(Box::new(DataType::Null)),
            dtype_names::STRUCT => DataType::Struct(vec![]),

            dtype_names::OP32 => DataType::Op32,
            dtype_names::NODE => DataType::Node(Box::new(DataType::Null)),

            _ => panic!("Unknown data type: {}", value),
        }
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Null => write!(f, "{}", dtype_names::NULL)?,

            DataType::UInt8 => write!(f, "{}", dtype_names::UINT8)?,
            DataType::UInt16 => write!(f, "{}", dtype_names::UINT16)?,
            DataType::UInt32 => write!(f, "{}", dtype_names::UINT32)?,
            DataType::UInt64 => write!(f, "{}", dtype_names::UINT64)?,
            DataType::UInt128 => write!(f, "{}", dtype_names::UINT128)?,

            DataType::Int8 => write!(f, "{}", dtype_names::INT8)?,
            DataType::Int16 => write!(f, "{}", dtype_names::INT16)?,
            DataType::Int32 => write!(f, "{}", dtype_names::INT32)?,
            DataType::Int64 => write!(f, "{}", dtype_names::INT64)?,
            DataType::Int128 => write!(f, "{}", dtype_names::INT128)?,

            DataType::Float32 => write!(f, "{}", dtype_names::FLOAT32)?,
            DataType::Float64 => write!(f, "{}", dtype_names::FLOAT64)?,

            DataType::Boolean => write!(f, "{}", dtype_names::BOOLEAN)?,

            DataType::Char => write!(f, "{}", dtype_names::CHAR)?,
            DataType::String => write!(f, "{}", dtype_names::STRING)?,

            DataType::List(inner) => write!(f, "{}({})", dtype_names::VEC, inner)?,
            DataType::Struct(vals) => write!(
                f,
                "struct({})",
                vals.iter()
                    .map(|f| format!("{}", f.name.clone()))
                    .collect::<Vec<_>>()
                    .join(", ")
            )?,
            DataType::Op32 => write!(f, "Op32")?,
            DataType::Node(inner) => write!(f, "Node({})", inner)?,
        };

        Ok(())
    }
}

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

    super::any_value_into_py_object(
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

    super::any_value_into_py_object(
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
        use crate::bindings::radiate;
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
            DataType::Op32 => {
                let class = rd.getattr(intern!(py, "Op32"))?;
                class.call0()
            }
            DataType::Node(inner) => {
                let class = rd.getattr(intern!(py, "Node"))?;
                let inner = Wrap(*inner.clone());
                class.call1((&inner,))
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
                    "Op32" => DataType::Op32,

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
            "Op32" => DataType::Op32,
            "Node" => {
                let inner = obj.getattr(intern!(py, "inner")).unwrap();
                let inner = inner.extract::<Wrap<DataType>>()?;
                DataType::Node(Box::new(inner.0))
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
