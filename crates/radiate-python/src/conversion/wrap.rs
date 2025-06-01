use crate::AnyValue;
use pyo3::{
    Bound, FromPyObject, IntoPyObject, PyAny, PyErr, PyResult, Python,
    exceptions::PyValueError,
    types::{PyAnyMethods, PyDict, PyList},
};
use radiate::{BlendCrossover, Chromosome, Gene, Metric, MetricSet, Optimize, Phenotype};

use super::{conversion::metric_to_py_dict, metric_set_to_py_dict};

/// # Safety
/// Should only be implemented for transparent types
#[allow(dead_code)]
pub(crate) unsafe trait Transparent {
    type Target;
}

unsafe impl<T> Transparent for Wrap<T> {
    type Target = T;
}

unsafe impl<T: Transparent> Transparent for Option<T> {
    type Target = Option<T::Target>;
}

#[repr(transparent)]
pub struct Wrap<T>(pub T);

impl<T> Clone for Wrap<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Wrap(self.0.clone())
    }
}
impl<T> From<T> for Wrap<T> {
    fn from(t: T) -> Self {
        Wrap(t)
    }
}

impl<'py> FromPyObject<'py> for Wrap<AnyValue<'py>> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
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
        self.clone().into_pyobject(py)
    }
}

impl<'py> FromPyObject<'py> for Wrap<Optimize> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let value = ob.extract::<String>()?;
        match value.as_str() {
            "min" => Ok(Wrap(Optimize::Minimize)),
            "max" => Ok(Wrap(Optimize::Maximize)),
            _ => Err(PyValueError::new_err(
                "Expected an Optimize value, but got a different type.",
            )),
        }
    }
}

/// Converts a `MetricSet` to a Python dictionary.
impl<'py> IntoPyObject<'py> for Wrap<MetricSet> {
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        metric_set_to_py_dict(py, &self.0).map_err(|e| {
            PyValueError::new_err(format!(
                "{e}\n\nHint: Try setting `strict=False` to allow passing data with mixed types."
            ))
        })
    }
}

/// Converts a `Metric` to a Python dictionary.
impl<'py> IntoPyObject<'py> for Wrap<Metric> {
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        metric_to_py_dict(py, &self.0).map_err(|e| {
            PyValueError::new_err(format!(
                "{e}\n\nHint: Try setting `strict=False` to allow passing data with mixed types."
            ))
        })
    }
}

impl<'py, G: Gene, C: Chromosome<Gene = G>> IntoPyObject<'py> for Wrap<Vec<Phenotype<C>>>
where
    G::Allele: Into<AnyValue<'static>> + Clone,
{
    type Target = PyList;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        super::pareto_front_to_py_object(py, &self.0).map_err(|e| {
            PyValueError::new_err(format!(
                "{e}\n\nHint: Try setting `strict=False` to allow passing data with mixed types."
            ))
        })
    }
}

// impl<'py> FromPyObject<'py> for Wrap<BlendCrossover> {
//     fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
//         let gene_type = ob.getattr("gene_type")?.extract::<String>()?;
//         let rate = ob.getattr("rate")?.extract::<f32>()?;
//         let alpha = ob.getattr("alpha")?.extract::<f32>()?;

//         if !(0.0..=1.0).contains(&rate) {
//             return Err(PyValueError::new_err("Rate must be between 0 and 1"));
//         }
//         if !(0.0..=1.0).contains(&alpha) {
//             return Err(PyValueError::new_err("Alpha must be between 0 and 1"));
//         }

//         Ok(Wrap(BlendCrossover::new(rate, alpha)))
//     }
// }
