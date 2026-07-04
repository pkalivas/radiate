use pyo3::{pyclass, pymethods};
use radiate::{Rate, rate::CycleShape};
use std::fmt::Debug;

use crate::PyExpr;

#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct PyRate {
    pub rate: Rate,
}

#[pymethods]
impl PyRate {
    pub fn value(&self, index: usize) -> f32 {
        self.rate.get_by_index(index)
    }

    #[staticmethod]
    pub fn fixed(value: f32) -> Self {
        PyRate {
            rate: Rate::Fixed(value),
        }
    }

    #[staticmethod]
    pub fn expression(expr: PyExpr) -> Self {
        PyRate {
            rate: Rate::Expr(expr.into()),
        }
    }
}

impl Debug for PyRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PyRate {{ rate: {:?} }}", self.rate)
    }
}
