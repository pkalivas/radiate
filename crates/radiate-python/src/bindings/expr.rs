use pyo3::prelude::*;
use radiate::{AnyValue, Evaluate, Expr};
use radiate_error::radiate_py_bail;

use crate::{PyMetricSet, Wrap, dtype_from_str};

fn dtype_is_duration(dtype_str: &str) -> bool {
    matches!(dtype_from_str(dtype_str), radiate::DataType::Duration)
}

#[pyclass(from_py_object)]
#[repr(transparent)]
#[derive(Clone)]
pub struct PyExpr {
    pub(crate) inner: Expr,
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
    #[pyo3(signature = (name, dtype=None))]
    pub fn select(name: &str, dtype: Option<&str>) -> Self {
        let mut e = Expr::select(name);
        if let Some(d) = dtype
            && dtype_is_duration(d)
        {
            e = e.time();
        }
        PyExpr { inner: e }
    }

    #[staticmethod]
    pub fn literal(value: Wrap<AnyValue<'_>>) -> Self {
        PyExpr {
            inner: Expr::lit(value.0.into_static()),
        }
    }

    #[staticmethod]
    pub fn when_then_otherwise(condition: &PyExpr, then_expr: &PyExpr, else_expr: &PyExpr) -> Self {
        PyExpr {
            inner: Expr::when(condition.inner().clone())
                .then(then_expr.inner().clone())
                .otherwise(else_expr.inner().clone()),
        }
    }

    #[staticmethod]
    pub fn every(interval: usize, then_expr: &PyExpr, else_expr: &PyExpr) -> Self {
        PyExpr {
            inner: Expr::every(interval)
                .then(then_expr.inner().clone())
                .otherwise(else_expr.inner().clone()),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (metric, epsilon=1e-4))]
    pub fn stagnation(metric: &str, epsilon: f32) -> Self {
        Expr::select(metric).stagnation(epsilon).into()
    }

    #[staticmethod]
    #[pyo3(signature = (metric, patience, epsilon=1e-4))]
    pub fn is_stagnant(metric: &str, patience: u32, epsilon: f32) -> Self {
        Expr::select(metric)
            .stagnation(epsilon)
            .gte(patience)
            .into()
    }

    pub fn evaluate(&mut self, metrics: &PyMetricSet) -> PyResult<Wrap<AnyValue<'_>>> {
        match self.inner.eval(metrics.inner()) {
            Ok(value) => Ok(Wrap(value)),
            Err(e) => {
                radiate_py_bail!(format!("Error evaluating expression: {}", e))
            }
        }
    }

    pub fn __repr__(&self) -> String {
        format!("{:#?}", self.inner)
    }

    pub fn __str__(&self) -> String {
        format!("{:?}", self.inner)
    }

    pub fn cast(&self, to: String) -> Self {
        let dtype = dtype_from_str(&to);
        self.inner.clone().cast(dtype).into()
    }

    pub fn debug(&self) -> Self {
        self.inner.clone().debug().into()
    }

    pub fn slope(&self) -> Self {
        self.inner.clone().slope().into()
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

    pub fn pow(&self, exp: &PyExpr) -> Self {
        self.inner.clone().pow(exp.inner.clone()).into()
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

    pub fn error(&self, target: f32) -> Self {
        self.inner.clone().error(target).into()
    }

    pub fn quantile(&self, q: f32) -> Self {
        if !(0.0..=1.0).contains(&q) {
            panic!("Quantile must be between 0 and 1");
        }

        self.inner.clone().quantile(q).into()
    }
}

impl From<Expr> for PyExpr {
    fn from(expr: Expr) -> Self {
        PyExpr { inner: expr }
    }
}

impl From<PyExpr> for Expr {
    fn from(py_expr: PyExpr) -> Self {
        py_expr.inner
    }
}
