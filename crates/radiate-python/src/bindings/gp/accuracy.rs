use crate::bindings::datatype::py_object_into_2d_vec;
use crate::{
    PyGraph, PyTree, Wrap,
    bindings::gp::{graph::PyGraphInner, tree::PyTreeInner},
};
use pyo3::{
    Bound, IntoPyObject, Py, PyAny, PyResult, Python, intern, pyclass, pyfunction, pymethods,
    types::PyAnyMethods,
};
use radiate::{Accuracy, AccuracyResult, DataSet, EvalMut, GraphEvaluator, Loss, ops::GpFloat};
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

    pub fn loss_fn<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        Wrap(self.inner.loss_fn()).into_pyobject(py)
    }
}

#[pyfunction]
#[pyo3(signature = (predictor, features, targets, loss=None, name=None))]
pub fn py_accuracy<'py>(
    py: Python<'py>,
    predictor: Py<PyAny>,
    features: &Bound<'py, PyAny>,
    targets: &Bound<'py, PyAny>,
    loss: Option<String>,
    name: Option<String>,
) -> PyResult<PyAccuracy> {
    let loss = parse_loss(loss)?;

    if let Ok(mut graph) = predictor.extract::<PyGraph>(py) {
        return match &mut graph.inner {
            PyGraphInner::Float32(graph, _) => {
                let features = py_object_into_2d_vec::<f32>(py, features)?;
                let targets = py_object_into_2d_vec::<f32>(py, targets)?;
                let mut evaluator = GraphEvaluator::new(graph);
                run_accuracy(&mut evaluator, features, targets, loss, name)
            }
            PyGraphInner::Float64(graph, _) => {
                let features = py_object_into_2d_vec::<f64>(py, features)?;
                let targets = py_object_into_2d_vec::<f64>(py, targets)?;
                let mut evaluator = GraphEvaluator::new(graph);
                run_accuracy(&mut evaluator, features, targets, loss, name)
            }
        };
    }

    if let Ok(mut tree) = predictor.extract::<PyTree>(py) {
        return match &mut tree.inner {
            PyTreeInner::Float32(trees) => {
                let features = py_object_into_2d_vec::<f32>(py, features)?;
                let targets = py_object_into_2d_vec::<f32>(py, targets)?;
                run_accuracy(trees, features, targets, loss, name)
            }
            PyTreeInner::Float64(trees) => {
                let features = py_object_into_2d_vec::<f64>(py, features)?;
                let targets = py_object_into_2d_vec::<f64>(py, targets)?;
                run_accuracy(trees, features, targets, loss, name)
            }
        };
    }

    radiate_py_bail!("Unsupported predictor type for accuracy calculation");
}

fn run_accuracy<F, E>(
    evaluator: &mut E,
    features: Vec<Vec<F>>,
    targets: Vec<Vec<F>>,
    loss: Loss,
    name: Option<String>,
) -> PyResult<PyAccuracy>
where
    F: GpFloat,
    E: EvalMut<[F], Vec<F>>,
{
    if features.len() != targets.len() {
        radiate_py_bail!("Accuracy: Features and targets must have the same number of samples");
    }

    let dataset = DataSet::new(features, targets);
    let mut accuracy = Accuracy::<F>::default().loss(loss);

    if let Some(name) = name {
        accuracy = accuracy.named(name);
    }

    let accuracy = accuracy.on(&dataset);
    let result = accuracy.calc(evaluator);

    Ok(PyAccuracy { inner: result })
}

fn parse_loss(loss: Option<String>) -> PyResult<Loss> {
    match loss {
        None => Ok(Loss::MSE),
        Some(loss) => match loss.trim().to_ascii_lowercase().as_str() {
            crate::constants::loss_functions::MSE_LOSS => Ok(Loss::MSE),
            crate::constants::loss_functions::MAE_LOSS => Ok(Loss::MAE),
            crate::constants::loss_functions::CROSS_ENTROPY_LOSS => Ok(Loss::XEnt),
            crate::constants::loss_functions::DIFF_LOSS => Ok(Loss::Diff),
            _ => {
                radiate_py_bail!("Unsupported loss function: {:loss?}");
            }
        },
    }
}

impl<'py> IntoPyObject<'py> for &Wrap<Loss> {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = pyo3::PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        use crate::bindings::radiate;

        let rd = radiate(py).bind(py);

        let class = match self.0 {
            Loss::MSE => rd.getattr(intern!(py, "MSE"))?,
            Loss::MAE => rd.getattr(intern!(py, "MAE"))?,
            Loss::XEnt => rd.getattr(intern!(py, "XEnt"))?,
            Loss::Diff => rd.getattr(intern!(py, "Diff"))?,
        };

        class.call0()
    }
}
