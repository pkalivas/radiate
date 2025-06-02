use crate::conversion::Wrap;
use pyo3::{
    Bound, FromPyObject, IntoPyObjectExt, PyAny, PyErr, PyResult, Python, pyclass, pymethods,
    types::{PyAnyMethods, PyString},
};
use radiate::{Objective, Optimize};
use std::vec;

const MIN: &str = "min";
const MAX: &str = "max";

#[pyclass(unsendable)]
#[derive(Clone, Debug)]
pub struct PyObjective {
    optimize: Vec<String>,
}

#[pymethods]
impl PyObjective {
    #[new]
    #[pyo3(signature = (optimize=None))]
    pub fn new(optimize: Option<Vec<String>>) -> PyResult<Self> {
        let optimize = match optimize {
            Some(opt) => opt,
            None => vec![MIN.to_string()],
        };

        if optimize.is_empty() || optimize.iter().any(|s| s != MIN && s != MAX) {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "At least one optimization direction must be specified",
            ));
        }

        Ok(PyObjective { optimize })
    }

    pub fn optimize(&self) -> Vec<String> {
        self.optimize.clone()
    }

    pub fn is_multi(&self) -> bool {
        self.optimize.len() > 1
    }

    pub fn is_single(&self) -> bool {
        self.optimize.len() == 1
    }

    pub fn __str__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.__repr__(py)
    }

    pub fn __repr__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let repr = format!("Objective(optimize={})", self.optimize.join(", "));
        PyString::new(py, &repr).into_bound_py_any(py)
    }

    #[staticmethod]
    pub fn max() -> PyResult<PyObjective> {
        Ok(PyObjective {
            optimize: vec![MAX.to_string()],
        })
    }

    #[staticmethod]
    pub fn min() -> PyResult<PyObjective> {
        Ok(PyObjective {
            optimize: vec![MIN.to_string()],
        })
    }

    #[staticmethod]
    pub fn multi(optimize: Vec<String>) -> PyResult<PyObjective> {
        if optimize.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "At least one optimization direction must be specified",
            ));
        }

        Ok(PyObjective { optimize })
    }
}

impl<'py> FromPyObject<'py> for Wrap<Objective> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        if let Ok(optimize) = ob.extract::<String>() {
            match optimize.as_str() {
                MIN => Ok(Wrap(Objective::Single(Optimize::Minimize))),
                MAX => Ok(Wrap(Objective::Single(Optimize::Maximize))),
                _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Invalid optimization direction. Use 'min' or 'max'.",
                )),
            }
        } else if let Ok(obj) = ob.extract::<Vec<String>>() {
            Ok(Wrap(Objective::Multi(
                obj.into_iter()
                    .map(|s| {
                        Ok(match s.as_str() {
                            MIN => Optimize::Minimize,
                            MAX => Optimize::Maximize,
                            _ => {
                                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                                    "Invalid optimization direction in list. Use 'min' or 'max'.",
                                ));
                            }
                        })
                    })
                    .filter_map(|opt| opt.ok())
                    .collect(),
            )))
        } else if let Ok(obj) = ob.extract::<PyObjective>() {
            // Convert PyObjective to Objective
            if obj.is_single() {
                Ok(Wrap(Objective::Single(match obj.optimize[0].as_str() {
                    MIN => Optimize::Minimize,
                    MAX => Optimize::Maximize,
                    _ => {
                        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            "Invalid optimization direction in PyObjective. Use 'min' or 'max'.",
                        ));
                    }
                })))
            } else if obj.is_multi() {
                Ok(Wrap(Objective::Multi(
                    obj.optimize
                        .into_iter()
                        .map(|s| Ok(match s.as_str() {
                            MIN => Optimize::Minimize,
                            MAX => Optimize::Maximize,
                            _ => {
                                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                                    "Invalid optimization direction in PyObjective. Use 'min' or 'max'.",
                                ));
                            }
                        }))
                        .filter_map(|opt| opt.ok())
                        .collect(),
                )))
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "PyObjective must have at least one optimization direction.",
                ))
            }
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected a string or an Objective instance.",
            ))
        }
    }
}
