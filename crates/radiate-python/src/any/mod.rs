mod arithmatic;
mod cell;
mod dtype;
mod gene;
mod serde;
mod time_unit;
mod time_zone;
pub(crate) mod value;

use cell::GILOnceCell;
use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, NaiveDateTime, TimeDelta, TimeZone};
use chrono_tz::Tz;
pub use dtype::Field;
pub use gene::{AnyChromosome, AnyGene};
use radiate::RadiateError;
pub use value::AnyValue;

use pyo3::{
    Borrowed, Bound, IntoPyObjectExt, Py, PyAny, PyResult, PyTypeCheck, Python,
    exceptions::{PyOverflowError, PyValueError},
    intern,
    types::{
        PyAnyMethods, PyBool, PyBytes, PyDate, PyDateTime, PyDict, PyDictMethods, PyFloat, PyInt,
        PyList, PySequence, PyString, PyTuple, PyType, PyTypeMethods, PyTzInfo,
    },
};
use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
    str::FromStr,
    sync::Arc,
};

type InitFn = for<'py> fn(&Bound<'py, PyAny>, bool) -> PyResult<AnyValue<'py>>;

pub(crate) static LUT: GILOnceCell<HashMap<TypeObjectKey, InitFn>> = GILOnceCell::new();

pub fn any_value_into_py_object_ref<'py, 'a>(
    av: &'a AnyValue<'a>,
    py: Python<'py>,
) -> PyResult<Bound<'py, PyAny>> {
    use AnyValue::*;
    match av {
        Null => py.None().into_bound_py_any(py),
        Bool(v) => v.into_bound_py_any(py),
        Char(v) => v.into_bound_py_any(py),
        UInt8(v) => v.into_bound_py_any(py),
        UInt16(v) => v.into_bound_py_any(py),
        UInt32(v) => v.into_bound_py_any(py),
        UInt64(v) => v.into_bound_py_any(py),
        Uint128(v) => v.into_bound_py_any(py),
        Int8(v) => v.into_bound_py_any(py),
        Int16(v) => v.into_bound_py_any(py),
        Int32(v) => v.into_bound_py_any(py),
        Int64(v) => v.into_bound_py_any(py),
        Int128(v) => v.into_bound_py_any(py),
        Float32(v) => v.into_bound_py_any(py),
        Float64(v) => v.into_bound_py_any(py),
        Str(s) => s.into_bound_py_any(py),
        StrOwned(s) => s.into_bound_py_any(py),
        Date(v) => v.into_bound_py_any(py),
        DateTime(v, tu, tz) => datetime_to_py_object(py, *v, *tu, tz.as_deref()),
        Binary(b) => pyo3::types::PyBytes::new(py, b).into_bound_py_any(py),
        Vector(v) => Ok(PyList::new(
            py,
            v.iter()
                .map(|item| any_value_into_py_object_ref(item, py))
                .collect::<PyResult<Vec<_>>>()?,
        )?
        .into_any()),
        Struct(pairs) => {
            let dict = pyo3::types::PyDict::new(py);
            for (fld, val) in pairs.iter() {
                let key = fld.name().to_string();
                let value = any_value_into_py_object_ref(val, py)?;
                dict.set_item(key, value)?;
            }
            Ok(dict.into_any())
        }
    }
}

pub fn any_value_into_py_object<'py>(av: AnyValue, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
    match av {
        AnyValue::UInt8(v) => v.into_bound_py_any(py),
        AnyValue::UInt16(v) => v.into_bound_py_any(py),
        AnyValue::UInt32(v) => v.into_bound_py_any(py),
        AnyValue::UInt64(v) => v.into_bound_py_any(py),
        AnyValue::Uint128(v) => v.into_bound_py_any(py),
        AnyValue::Int8(v) => v.into_bound_py_any(py),
        AnyValue::Int16(v) => v.into_bound_py_any(py),
        AnyValue::Int32(v) => v.into_bound_py_any(py),
        AnyValue::Int64(v) => v.into_bound_py_any(py),
        AnyValue::Int128(v) => v.into_bound_py_any(py),
        AnyValue::Float32(v) => v.into_bound_py_any(py),
        AnyValue::Float64(v) => v.into_bound_py_any(py),
        AnyValue::Char(v) => v.into_bound_py_any(py),
        AnyValue::Date(v) => v.into_bound_py_any(py),
        AnyValue::DateTime(v, tu, tz) => datetime_to_py_object(py, v, tu, tz.as_deref()),
        AnyValue::Vector(v) => Ok(PyList::new(
            py,
            v.into_iter()
                .map(|item| any_value_into_py_object(item, py))
                .collect::<PyResult<Vec<_>>>()?,
        )?
        .into_any()),
        AnyValue::Null => py.None().into_bound_py_any(py),
        AnyValue::Bool(v) => v.into_bound_py_any(py),
        AnyValue::Str(v) => v.into_bound_py_any(py),
        AnyValue::StrOwned(v) => v.into_bound_py_any(py),
        AnyValue::Binary(v) => PyBytes::new(py, &v).into_bound_py_any(py),
        AnyValue::Struct(v) => {
            let dict = struct_dict(py, v.into_iter())?;
            dict.into_bound_py_any(py)
        }
    }
}

pub fn elapsed_offset_to_timedelta(elapsed: i64, time_unit: time_unit::TimeUnit) -> TimeDelta {
    let (in_second, nano_multiplier) = match time_unit {
        time_unit::TimeUnit::Nanoseconds => (1_000_000_000, 1),
        time_unit::TimeUnit::Microseconds => (1_000_000, 1_000),
        time_unit::TimeUnit::Milliseconds => (1_000, 1_000_000),
    };
    let mut elapsed_sec = elapsed / in_second;
    let mut elapsed_nanos = nano_multiplier * (elapsed % in_second);
    if elapsed_nanos < 0 {
        // TimeDelta expects nanos to always be positive.
        elapsed_sec -= 1;
        elapsed_nanos += 1_000_000_000;
    }
    TimeDelta::new(elapsed_sec, elapsed_nanos as u32).unwrap()
}

pub fn timestamp_to_naive_datetime(
    since_epoch: i64,
    time_unit: time_unit::TimeUnit,
) -> NaiveDateTime {
    DateTime::UNIX_EPOCH.naive_utc() + elapsed_offset_to_timedelta(since_epoch, time_unit)
}

pub fn datetime_to_py_object<'py>(
    py: Python<'py>,
    v: i64,
    tu: time_unit::TimeUnit,
    tz: Option<&time_zone::TimeZone>,
) -> PyResult<Bound<'py, PyAny>> {
    if let Some(time_zone) = tz {
        if let Ok(tz) = Tz::from_str(time_zone) {
            let utc_datetime = DateTime::UNIX_EPOCH + elapsed_offset_to_timedelta(v, tu);
            if utc_datetime.year() >= 2100 {
                // chrono-tz does not support dates after 2100
                // https://github.com/chronotope/chrono-tz/issues/135
                let datetime = utc_datetime.naive_utc();
                datetime.into_bound_py_any(py)
            } else {
                let datetime = utc_datetime.with_timezone(&tz);
                datetime.into_bound_py_any(py)
            }
        } else if let Ok(tz) = FixedOffset::from_str(time_zone) {
            let naive_datetime = timestamp_to_naive_datetime(v, tu);
            let datetime = tz.from_utc_datetime(&naive_datetime);
            datetime.into_bound_py_any(py)
        } else {
            Err(RadiateError::Other(format!("Could not parse timezone: {time_zone}")).into())
        }
    } else {
        timestamp_to_naive_datetime(v, tu).into_bound_py_any(py)
    }
}

pub fn py_object_to_any_value<'a, 'py>(
    ob: Borrowed<'a, 'py, PyAny>,
    strict: bool,
) -> PyResult<AnyValue<'py>> {
    fn get_null(_ob: &Bound<'_, PyAny>, _strict: bool) -> PyResult<AnyValue<'static>> {
        Ok(AnyValue::Null)
    }

    fn get_bool(ob: &Bound<'_, PyAny>, _strict: bool) -> PyResult<AnyValue<'static>> {
        let b = ob.extract::<bool>()?;
        Ok(AnyValue::Bool(b))
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
        Ok(AnyValue::StrOwned(ob.extract::<String>()?.into()))
    }

    fn get_bytes<'py>(ob: &Bound<'py, PyAny>, _strict: bool) -> PyResult<AnyValue<'py>> {
        let value = ob.extract::<Vec<u8>>()?;
        Ok(AnyValue::Binary(value))
    }

    fn get_date(ob: &Bound<'_, PyAny>, _strict: bool) -> PyResult<AnyValue<'static>> {
        const UNIX_EPOCH: NaiveDate = DateTime::UNIX_EPOCH.naive_utc().date();
        let date = ob.extract::<NaiveDate>()?;
        let elapsed = date.signed_duration_since(UNIX_EPOCH);
        Ok(AnyValue::Date(elapsed.num_days() as i32))
    }

    fn get_datetime(ob: &Bound<'_, PyAny>, _strict: bool) -> PyResult<AnyValue<'static>> {
        let py = ob.py();
        let tzinfo = ob.getattr(intern!(py, "tzinfo"))?;

        if tzinfo.is_none() {
            let datetime = ob.extract::<NaiveDateTime>()?;
            let delta = datetime - DateTime::UNIX_EPOCH.naive_utc();
            let timestamp = delta.num_microseconds().unwrap();
            return Ok(AnyValue::DateTime(
                timestamp,
                time_unit::TimeUnit::Microseconds,
                None,
            ));
        }

        // Try converting `pytz` timezone to `zoneinfo` timezone
        let (ob, tzinfo) = if let Some(tz) = tzinfo
            .getattr(intern!(py, "zone"))
            .ok()
            .and_then(|tz| (!tz.is_none()).then_some(tz))
        {
            let tzinfo = PyTzInfo::timezone(py, tz.cast::<PyString>()?)?;
            (
                &ob.call_method(intern!(py, "astimezone"), (&tzinfo,), None)?,
                tzinfo,
            )
        } else {
            (ob, tzinfo.cast_into()?)
        };

        let (timestamp, tz) = if tzinfo.hasattr(intern!(py, "key"))? {
            let datetime = ob.extract::<DateTime<Tz>>()?;
            let tz = unsafe { time_zone::TimeZone::from_static(datetime.timezone().name()) };
            if datetime.year() >= 2100 {
                // chrono-tz does not support dates after 2100
                // https://github.com/chronotope/chrono-tz/issues/135
                let delta = datetime.to_utc() - DateTime::UNIX_EPOCH;
                (delta.num_microseconds().unwrap(), tz)
            } else {
                let delta = datetime.to_utc() - DateTime::UNIX_EPOCH;
                (delta.num_microseconds().unwrap(), tz)
            }
        } else {
            let datetime = ob.extract::<DateTime<FixedOffset>>()?;
            let delta = datetime.to_utc() - DateTime::UNIX_EPOCH;
            (delta.num_microseconds().unwrap(), time_zone::TimeZone::UTC)
        };

        Ok(AnyValue::DateTime(
            timestamp,
            time_unit::TimeUnit::Microseconds,
            Some(Arc::new(tz)),
        ))
    }

    fn get_list(ob: &Bound<'_, PyAny>, strict: bool) -> PyResult<AnyValue<'static>> {
        let seq = ob.cast::<PySequence>()?;
        let mut out: Vec<AnyValue<'static>> = Vec::with_capacity(seq.len().unwrap_or(0).max(0));
        for item in seq.try_iter()? {
            let av = py_object_to_any_value(item?.as_borrowed(), strict)?;
            out.push(av.into_static());
        }
        Ok(AnyValue::Vector(Box::new(out)))
    }

    fn get_conversion_function(ob: &Bound<'_, PyAny>, strict: bool) -> PyResult<InitFn> {
        if ob.is_none() {
            Ok(get_null)
        } else if ob.is_instance_of::<PyBool>() {
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
        } else if PyDateTime::type_check(ob) {
            Ok(get_datetime as InitFn)
        } else if PyDate::type_check(ob) {
            Ok(get_date as InitFn)
        } else if ob.is_instance_of::<PyDict>() {
            Ok(|ob, strict| {
                let dict = ob.cast::<PyDict>().unwrap();
                let len = dict.len();
                let mut key_value_pairs = Vec::with_capacity(len);
                for (k, v) in dict.into_iter() {
                    let key = k.extract::<Cow<str>>()?;
                    let val = py_object_to_any_value(v.as_borrowed(), strict)?;
                    key_value_pairs.push((Field::new(key.as_ref().into()), val));
                }

                Ok(AnyValue::Struct(key_value_pairs))
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

    let py = ob.py();

    if !LUT.is_initialized() {
        LUT.set(py, Default::default()).unwrap();
    }

    LUT.with_gil(py, |lut| {
        if !lut.contains_key(&py_type_address) {
            let k = TypeObjectKey::new(py_type.clone().unbind());

            assert_eq!(k.address, py_type_address);

            lut.insert(k, get_conversion_function(ob.as_any(), strict)?);
        }

        let conversion_func = lut.get(&py_type_address).unwrap();
        conversion_func(ob.as_any(), strict)
    })
}

fn struct_dict<'py, 'a>(
    py: Python<'py>,
    vals: impl Iterator<Item = (Field, AnyValue<'a>)>,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);

    for (fld, val) in vals {
        let key = fld.name().to_string();
        let value = any_value_into_py_object(val, py)?;
        dict.set_item(key, value)?;
    }

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
