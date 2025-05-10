use super::Wrap;
use crate::object::{AnyValue, Field};
use pyo3::{
    Bound, IntoPyObject, IntoPyObjectExt, Py, PyAny, PyResult, Python,
    exceptions::{PyOverflowError, PyValueError},
    types::{
        PyAnyMethods, PyBool, PyBytes, PyDict, PyDictMethods, PyFloat, PyInt, PyList,
        PyListMethods, PySequence, PyString, PyTuple, PyType, PyTypeMethods,
    },
};
use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
};

type InitFn = for<'py> fn(&Bound<'py, PyAny>, bool) -> PyResult<AnyValue<'py>>;
pub(crate) static LUT: crate::GILOnceCell<HashMap<TypeObjectKey, InitFn>> =
    crate::GILOnceCell::new();

pub fn any_value_into_py_object<'py>(av: AnyValue, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
    match av {
        AnyValue::UInt8(v) => v.into_bound_py_any(py),
        AnyValue::UInt16(v) => v.into_bound_py_any(py),
        AnyValue::UInt32(v) => v.into_bound_py_any(py),
        AnyValue::UInt64(v) => v.into_bound_py_any(py),
        AnyValue::Int8(v) => v.into_bound_py_any(py),
        AnyValue::Int16(v) => v.into_bound_py_any(py),
        AnyValue::Int32(v) => v.into_bound_py_any(py),
        AnyValue::Int64(v) => v.into_bound_py_any(py),
        AnyValue::Int128(v) => v.into_bound_py_any(py),
        AnyValue::Float32(v) => v.into_bound_py_any(py),
        AnyValue::Float64(v) => v.into_bound_py_any(py),
        AnyValue::Char(v) => v.into_bound_py_any(py),
        AnyValue::Slice(v, _) => {
            let list = PyList::empty(py);
            for item in v.iter() {
                list.append(any_value_into_py_object(item.clone(), py)?)?;
            }
            Ok(list.into_any())
        }
        AnyValue::VecOwned(v) => {
            let list = PyList::empty(py);
            for item in v.0.into_iter() {
                list.append(any_value_into_py_object(item, py)?)?;
            }
            Ok(list.into_any())
        }
        AnyValue::Null => py.None().into_bound_py_any(py),
        AnyValue::Boolean(v) => v.into_bound_py_any(py),
        AnyValue::String(v) => v.into_bound_py_any(py),
        AnyValue::StringOwned(v) => v.into_bound_py_any(py),
        AnyValue::Binary(v) => PyBytes::new(py, v).into_bound_py_any(py),
        AnyValue::BinaryOwned(v) => PyBytes::new(py, &v).into_bound_py_any(py),
        AnyValue::StructOwned(v) => {
            let (vals, flds) = *v;
            let dict = struct_dict(py, vals.into_iter(), &flds)?;
            dict.into_bound_py_any(py)
        }
    }
}

/// Convert a Python object to an [`AnyValue`].
pub fn py_object_to_any_value<'py>(
    ob: &Bound<'py, PyAny>,
    strict: bool,
) -> PyResult<AnyValue<'py>> {
    // Conversion functions.
    fn get_null(_ob: &Bound<'_, PyAny>, _strict: bool) -> PyResult<AnyValue<'static>> {
        Ok(AnyValue::Null)
    }

    fn get_bool(ob: &Bound<'_, PyAny>, _strict: bool) -> PyResult<AnyValue<'static>> {
        let b = ob.extract::<bool>()?;
        Ok(AnyValue::Boolean(b))
    }

    fn get_int(ob: &Bound<'_, PyAny>, strict: bool) -> PyResult<AnyValue<'static>> {
        if let Ok(v) = ob.extract::<i64>() {
            Ok(AnyValue::Int64(v))
        } else if let Ok(v) = ob.extract::<i128>() {
            Ok(AnyValue::Int128(v))
        } else if !strict {
            let f = ob.extract::<f64>()?;
            Ok(AnyValue::Float64(f))
        } else {
            Err(PyOverflowError::new_err(format!(
                "int value too large for Polars integer types: {ob}"
            )))
        }
    }

    fn get_float(ob: &Bound<'_, PyAny>, _strict: bool) -> PyResult<AnyValue<'static>> {
        Ok(AnyValue::Float64(ob.extract::<f64>()?))
    }

    fn get_str(ob: &Bound<'_, PyAny>, _strict: bool) -> PyResult<AnyValue<'static>> {
        // Ideally we'd be returning an AnyValue::String(&str) instead, as was
        // the case in previous versions of this function. However, if compiling
        // with abi3 for versions older than Python 3.10, the APIs that purport
        // to return &str actually just encode to UTF-8 as a newly allocated
        // PyBytes object, and then return reference to that. So what we're
        // doing here isn't any different fundamentally, and the APIs to for
        // converting to &str are deprecated in PyO3 0.21.
        //
        // Once Python 3.10 is the minimum supported version, converting to &str
        // will be cheaper, and we should do that. Python 3.9 security updates
        // end-of-life is Oct 31, 2025.
        Ok(AnyValue::StringOwned(ob.extract::<String>()?.into()))
    }

    fn get_bytes<'py>(ob: &Bound<'py, PyAny>, _strict: bool) -> PyResult<AnyValue<'py>> {
        let value = ob.extract::<Vec<u8>>()?;
        Ok(AnyValue::BinaryOwned(value))
    }

    fn get_list(ob: &Bound<'_, PyAny>, strict: bool) -> PyResult<AnyValue<'static>> {
        if ob.is_empty()? {
            Ok(AnyValue::Null)
        } else if ob.is_instance_of::<PyList>() | ob.is_instance_of::<PyTuple>() {
            const INFER_SCHEMA_LENGTH: usize = 25;

            let list = ob.downcast::<PySequence>()?;

            let mut avs = Vec::with_capacity(INFER_SCHEMA_LENGTH);
            let mut iter = list.try_iter()?;
            let mut items = Vec::with_capacity(INFER_SCHEMA_LENGTH);
            for item in (&mut iter).take(INFER_SCHEMA_LENGTH) {
                items.push(item?);
                let av = py_object_to_any_value(items.last().unwrap(), strict)?;
                avs.push(av)
            }

            Ok(AnyValue::Null)
        } else if !strict {
            Ok(AnyValue::Null)
        } else {
            Err(PyValueError::new_err(format!(
                "Cannot convert Python object of type {} to AnyValue",
                ob.get_type().qualname()?
            )))
        }
    }

    /// Determine which conversion function to use for the given object.
    ///
    /// Note: This function is only ran if the object's type is not already in the
    /// lookup table.
    fn get_conversion_function(ob: &Bound<'_, PyAny>, strict: bool) -> PyResult<InitFn> {
        if ob.is_none() {
            Ok(get_null)
        }
        // bool must be checked before int because Python bool is an instance of int.
        else if ob.is_instance_of::<PyBool>() {
            Ok(get_bool)
        } else if ob.is_instance_of::<PyInt>() {
            Ok(get_int)
        } else if ob.is_instance_of::<PyFloat>() {
            Ok(get_float)
        } else if ob.is_instance_of::<PyString>() {
            Ok(get_str)
        } else if ob.is_instance_of::<PyBytes>() {
            Ok(get_bytes)
        } else if ob.is_instance_of::<PyList>() || ob.is_instance_of::<PyTuple>() {
            Ok(get_list)
        } else if ob.is_instance_of::<PyDict>() {
            Ok(|ob, strict| {
                let dict = ob.downcast::<PyDict>().unwrap();
                let len = dict.len();
                let mut keys = Vec::with_capacity(len);
                let mut vals = Vec::with_capacity(len);
                for (k, v) in dict.into_iter() {
                    let key = k.extract::<Cow<str>>()?;
                    let val = py_object_to_any_value(&v, strict)?;
                    let dtype = val.dtype();
                    keys.push(Field::new(key.as_ref().into(), dtype));
                    vals.push(val)
                }
                Ok(AnyValue::StructOwned(Box::new((vals, keys))))
            })
        } else {
            let ob_type = ob.get_type();
            let type_name = ob_type.qualname()?.to_string();
            match type_name.as_str() {
                "range" => Ok(get_list as InitFn),
                _ => {
                    if !strict {
                        Ok(get_null)
                    } else {
                        Err(PyValueError::new_err(format!(
                            "Cannot convert Python object of type {type_name} to AnyValue"
                        )))
                    }
                }
            }
        }
    }

    let py_type = ob.get_type();
    let py_type_address = py_type.as_ptr() as usize;

    Python::with_gil(|py| {
        if !LUT.is_initialized() {
            LUT.set(py, Default::default()).unwrap();
        }

        LUT.with_gil(py, |lut| {
            if !lut.contains_key(&py_type_address) {
                let k = TypeObjectKey::new(py_type.clone().unbind());

                assert_eq!(k.address, py_type_address);

                lut.insert(k, get_conversion_function(ob, strict)?);
            }

            let conversion_func = lut.get(&py_type_address).unwrap();
            conversion_func(ob, strict)
        })
    })
}

fn struct_dict<'py, 'a>(
    py: Python<'py>,
    vals: impl Iterator<Item = AnyValue<'a>>,
    flds: &[Field],
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    flds.iter().zip(vals).try_for_each(|(fld, val)| {
        dict.set_item(fld.name().as_str(), Wrap(val).into_pyobject(py)?)
    })?;
    Ok(dict)
}

/// Holds a Python type object and implements hashing / equality based on the pointer address of the
/// type object. This is used as a hashtable key instead of only the `usize` pointer value, as we
/// need to hold a ref to the Python type object to keep it alive.
#[derive(Debug)]
pub struct TypeObjectKey {
    #[allow(unused)]
    type_object: Py<PyType>,
    /// We need to store this in a field for `Borrow<usize>`
    address: usize,
}

impl TypeObjectKey {
    fn new(type_object: Py<PyType>) -> Self {
        let address = type_object.as_ptr() as usize;
        Self {
            type_object,
            address,
        }
    }
}

impl PartialEq for TypeObjectKey {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}

impl Eq for TypeObjectKey {}

impl std::borrow::Borrow<usize> for TypeObjectKey {
    fn borrow(&self) -> &usize {
        &self.address
    }
}

impl std::hash::Hash for TypeObjectKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let v: &usize = self.borrow();
        v.hash(state)
    }
}
