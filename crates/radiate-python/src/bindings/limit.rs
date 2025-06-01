use crate::conversion::Wrap;
use pyo3::{
    Bound, FromPyObject, PyAny, PyErr, PyResult, pyclass, pymethods,
    types::{PyAnyMethods, PyList, PyListMethods},
};

#[pyclass(unsendable)]
#[derive(Clone, Debug)]
pub enum PyLimit {
    Generation(usize),
    Seconds(f64),
    Score(f32),
}

#[pymethods]
impl PyLimit {
    pub fn __str__(&self) -> String {
        match self {
            PyLimit::Generation(generation) => format!("Generation({})", generation),
            PyLimit::Seconds(sec) => format!("Seconds({})", sec),
            PyLimit::Score(score) => format!("Score({})", score),
        }
    }

    pub fn __repr__(&self) -> String {
        self.__str__()
    }
}

impl<'py> FromPyObject<'py> for Wrap<Vec<PyLimit>> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let limits = if let Ok(limits) = ob.extract::<Vec<PyLimit>>() {
            limits
        } else if let Ok(limits) = ob.extract::<Bound<'_, PyList>>() {
            limits
                .iter()
                .map(|l| l.extract::<PyLimit>())
                .collect::<PyResult<Vec<_>>>()?
        } else if let Ok(limit) = ob.extract::<PyLimit>() {
            vec![limit]
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected a list of limits",
            ));
        };

        if limits.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "At least one limit must be specified",
            ));
        }

        Ok(Wrap(limits))
    }
}
