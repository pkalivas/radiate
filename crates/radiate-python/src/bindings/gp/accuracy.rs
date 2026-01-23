use crate::{PyGraph, PyTree};
use pyo3::{Py, PyAny, PyResult, Python, pyclass, pyfunction, pymethods};
use radiate::{Accuracy, AccuracyResult, DataSet, Eval, Loss};
use radiate_error::radiate_py_bail;

#[pyclass]
pub struct PyAccuracy {
    inner: AccuracyResult,
}

#[pymethods]
impl PyAccuracy {
    pub fn __repr__(&self) -> String {
        format!("PyAccuracy({:?})", self.inner)
    }

    pub fn __str__(&self) -> String {
        format!("{:?}", self.inner)
    }

    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }

    pub fn accuracy(&self) -> f32 {
        self.inner.accuracy()
    }

    pub fn precision(&self) -> f32 {
        self.inner.precision()
    }

    pub fn recall(&self) -> f32 {
        self.inner.recall()
    }

    pub fn f1_score(&self) -> f32 {
        self.inner.f1_score()
    }

    pub fn rmse(&self) -> f32 {
        self.inner.rmse()
    }

    pub fn r_squared(&self) -> f32 {
        self.inner.r_squared()
    }

    pub fn sample_count(&self) -> usize {
        self.inner.sample_count()
    }

    pub fn loss(&self) -> f32 {
        self.inner.loss()
    }

    pub fn loss_fn(&self) -> String {
        format!("{:?}", self.inner.loss_fn())
    }
}

#[pyfunction]
#[pyo3(signature = (predictor, features, targets, loss=None, name=None))]
pub fn py_accuracy<'py>(
    py: Python<'py>,
    predictor: Py<PyAny>,
    features: Vec<Vec<f32>>,
    targets: Vec<Vec<f32>>,
    loss: Option<String>,
    name: Option<String>,
) -> PyResult<PyAccuracy> {
    if !features.len().eq(&targets.len()) {
        radiate_py_bail!("Accuracy: Features and targets must have the same number of samples");
    }

    let data_set = DataSet::new(features, targets);
    let loss = match loss {
        Some(loss_name) => match loss_name.as_str() {
            crate::names::MSE_LOSS => Loss::MSE,
            crate::names::MAE_LOSS => Loss::MAE,
            crate::names::CROSS_ENTROPY_LOSS => Loss::CrossEntropy,
            crate::names::DIFF_LOSS => Loss::Diff,
            _ => panic!("Unsupported loss function: {}", loss_name),
        },
        None => Loss::MSE,
    };

    let accuracy = Accuracy::new(name.unwrap_or_default())
        .loss(loss)
        .on(&data_set);

    if let Ok(graph) = predictor.extract::<PyGraph>(py) {
        match accuracy.eval(&graph.inner) {
            Some(result) => Ok(PyAccuracy { inner: result }),
            None => radiate_py_bail!("Accuracy calculation for Graph failed during evaluation"),
        }
    } else if let Ok(tree) = predictor.extract::<PyTree>(py) {
        match accuracy.eval(&tree.inner) {
            Some(result) => Ok(PyAccuracy { inner: result }),
            None => radiate_py_bail!("Accuracy calculation for Tree failed during evaluation"),
        }
    } else {
        radiate_py_bail!("Unsupported predictor type for accuracy calculation");
    }
}
