use pyo3::prelude::*;
use radiate::{AnyValue, Expr, ExprQuery, expr};

use crate::Wrap;

#[pyclass(from_py_object)]
#[repr(transparent)]
#[derive(Clone)]
pub struct PyExpr {
    inner: Expr,
}

impl PyExpr {
    pub fn inner_mut(&mut self) -> &mut Expr {
        &mut self.inner
    }

    pub fn inner(&self) -> &Expr {
        &self.inner
    }
}

#[pymethods]
impl PyExpr {
    #[staticmethod]
    pub fn metric(name: &str) -> Self {
        PyExpr {
            inner: expr::select(name),
        }
    }

    #[staticmethod]
    pub fn literal(value: Wrap<AnyValue<'_>>) -> Self {
        PyExpr {
            inner: expr::lit(value.0.into_static()),
        }
    }

    #[staticmethod]
    pub fn when_then_otherwise(condition: &PyExpr, then_expr: &PyExpr, else_expr: &PyExpr) -> Self {
        PyExpr {
            inner: expr::when(condition.inner().clone())
                .then(then_expr.inner().clone())
                .otherwise(else_expr.inner().clone()),
        }
    }

    #[staticmethod]
    pub fn every(interval: usize, then_expr: &PyExpr, else_expr: &PyExpr) -> Self {
        PyExpr {
            inner: expr::every(interval)
                .then(then_expr.inner().clone())
                .otherwise(else_expr.inner().clone()),
        }
    }

    #[staticmethod]
    pub fn element() -> Self {
        PyExpr {
            inner: expr::element(),
        }
    }

    pub fn evaluate<'py>(&mut self, input: Wrap<AnyValue<'_>>) -> PyResult<Wrap<AnyValue<'_>>> {
        let result = input.0.into_static();
        match self.inner.dispatch(&result) {
            Ok(value) => return Ok(Wrap(value)),
            Err(e) => {
                let msg = format!("Error evaluating expression: {}", e);
                return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(msg));
            }
        }
    }

    pub fn __repr__(&self) -> String {
        format!("{:#?}", self.inner)
    }

    pub fn __str__(&self) -> String {
        format!("{:?}", self.inner)
    }

    pub fn time(&self) -> Self {
        self.inner.clone().time().into()
    }

    pub fn rolling(&self, window: usize) -> Self {
        self.inner.clone().rolling(window).into()
    }

    pub fn first(&self) -> Self {
        self.inner.clone().first().into()
    }

    pub fn last(&self) -> Self {
        self.inner.clone().last().into()
    }

    pub fn sum(&self) -> Self {
        self.inner.clone().sum().into()
    }

    pub fn mean(&self) -> Self {
        self.inner.clone().mean().into()
    }

    pub fn stddev(&self) -> Self {
        self.inner.clone().stddev().into()
    }

    pub fn min(&self) -> Self {
        self.inner.clone().min().into()
    }

    pub fn max(&self) -> Self {
        self.inner.clone().max().into()
    }

    pub fn var(&self) -> Self {
        self.inner.clone().var().into()
    }

    pub fn skew(&self) -> Self {
        self.inner.clone().skew().into()
    }

    pub fn count(&self) -> Self {
        self.inner.clone().count().into()
    }

    pub fn unique(&self) -> Self {
        self.inner.clone().unique().into()
    }

    pub fn lt(&self, rhs: &PyExpr) -> Self {
        self.inner.clone().lt(rhs.inner.clone()).into()
    }

    pub fn lte(&self, rhs: &PyExpr) -> Self {
        self.inner.clone().lte(rhs.inner.clone()).into()
    }

    pub fn gt(&self, rhs: &PyExpr) -> Self {
        self.inner.clone().gt(rhs.inner.clone()).into()
    }

    pub fn gte(&self, rhs: &PyExpr) -> Self {
        self.inner.clone().gte(rhs.inner.clone()).into()
    }

    pub fn eq(&self, rhs: &PyExpr) -> Self {
        self.inner.clone().eq(rhs.inner.clone()).into()
    }

    pub fn ne(&self, rhs: &PyExpr) -> Self {
        self.inner.clone().ne(rhs.inner.clone()).into()
    }

    pub fn and_(&self, rhs: &PyExpr) -> Self {
        self.inner.clone().and(rhs.inner.clone()).into()
    }

    pub fn or_(&self, rhs: &PyExpr) -> Self {
        self.inner.clone().or(rhs.inner.clone()).into()
    }

    pub fn not_(&self) -> Self {
        self.inner.clone().not().into()
    }

    pub fn neg_(&self) -> Self {
        self.inner.clone().neg().into()
    }

    pub fn abs_(&self) -> Self {
        self.inner.clone().abs().into()
    }

    pub fn add_(&self, rhs: &PyExpr) -> Self {
        self.inner.clone().add(rhs.inner.clone()).into()
    }

    pub fn sub_(&self, rhs: &PyExpr) -> Self {
        self.inner.clone().sub(rhs.inner.clone()).into()
    }

    pub fn mul_(&self, rhs: &PyExpr) -> Self {
        self.inner.clone().mul(rhs.inner.clone()).into()
    }

    pub fn div_(&self, rhs: &PyExpr) -> Self {
        self.inner.clone().div(rhs.inner.clone()).into()
    }

    pub fn clamp_(&self, min: &PyExpr, max: &PyExpr) -> Self {
        self.inner
            .clone()
            .clamp(min.inner.clone(), max.inner.clone())
            .into()
    }
}

impl From<Expr> for PyExpr {
    fn from(expr: Expr) -> Self {
        PyExpr { inner: expr }
    }
}

impl Into<Expr> for PyExpr {
    fn into(self) -> Expr {
        self.inner
    }
}
