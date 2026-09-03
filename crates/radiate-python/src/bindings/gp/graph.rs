use crate::{IntoPyAnyObject, PyAnyObject, Wrap};
use numpy::PyArrayDyn;
use pyo3::{
    Bound, IntoPyObject, IntoPyObjectExt, Py, PyAny, PyResult, Python, intern,
    prelude::FromPyObjectOwned, pyclass, pymethods, sync::PyOnceLock, types::PyAnyMethods,
};
use pyo3::{
    BoundObject,
    types::{PyBytes, PyBytesMethods},
};
use radiate::{DataType, EvalMut, Graph, Op, RadiateResult, StatefulGraph, ToDot};
use radiate_utils::Float;
use serde::{Deserialize, Serialize};

static GRAPH_FROM_RUST: PyOnceLock<Py<PyAny>> = PyOnceLock::new();

fn graph_from_rust(py: Python<'_>) -> &Py<PyAny> {
    GRAPH_FROM_RUST.get_or_init(py, || {
        use crate::bindings::radiate;
        radiate(py)
            .bind(py)
            .getattr(intern!(py, "Graph"))
            .unwrap()
            .getattr(intern!(py, "from_rust"))
            .unwrap()
            .unbind()
    })
}

#[inline]
fn eval_graph<'py, F>(
    py: Python<'py>,
    graph: &mut StatefulGraph<Op<F>, F>,
    output_len: usize,
    inputs: &Bound<'py, PyAny>,
) -> PyResult<Bound<'py, PyArrayDyn<F>>>
where
    F: Float + numpy::Element + FromPyObjectOwned<'py>,
{
    super::generic_eval_runner(py, output_len, inputs, |slice| graph.eval_mut(slice))
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub(crate) enum PyGraphInner {
    Float32(StatefulGraph<Op<f32>, f32>),
    Float64(StatefulGraph<Op<f64>, f64>),
}

#[pyclass(from_py_object)]
#[derive(Serialize, Deserialize)]
pub struct PyGraph {
    pub(crate) inner: PyGraphInner,
}

#[pymethods]
impl PyGraph {
    #[staticmethod]
    pub fn from_json(json: &str) -> PyResult<Self> {
        serde_json::from_str::<PyGraph>(json)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid JSON: {}", e)))
    }

    #[staticmethod]
    pub fn from_pickle<'py>(pickle_bytes: &Bound<'py, PyBytes>) -> PyResult<Self> {
        serde_pickle::from_slice::<PyGraph>(
            pickle_bytes.as_bytes(),
            serde_pickle::DeOptions::default(),
        )
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid Pickle: {}", e)))
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn to_pickle<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let pickle =
            serde_pickle::to_vec(self, serde_pickle::SerOptions::default()).map_err(|e| {
                pyo3::exceptions::PyValueError::new_err(format!(
                    "Failed to serialize to pickle: {}",
                    e
                ))
            })?;

        Ok(PyBytes::new(python, &pickle).into_bound())
    }

    pub fn to_dot(&self) -> String {
        match &self.inner {
            PyGraphInner::Float32(graph) => graph.as_ref().to_dot(),
            PyGraphInner::Float64(graph) => graph.as_ref().to_dot(),
        }
    }

    pub fn dtype<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let result = match &self.inner {
            PyGraphInner::Float32(_) => DataType::Float32,
            PyGraphInner::Float64(_) => DataType::Float64,
        };

        Wrap(result).into_pyobject(py)
    }

    pub fn reset(&mut self) {
        match &mut self.inner {
            PyGraphInner::Float32(graph) => graph.reset(),
            PyGraphInner::Float64(graph) => graph.reset(),
        }
    }

    pub fn shape(&self) -> (usize, usize) {
        match &self.inner {
            PyGraphInner::Float32(graph) => (graph.input_dim(), graph.output_dim()),
            PyGraphInner::Float64(graph) => (graph.input_dim(), graph.output_dim()),
        }
    }

    #[inline]
    pub fn eval<'py>(
        &mut self,
        py: Python<'py>,
        inputs: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let output_len = self.shape().1;
        match &mut self.inner {
            PyGraphInner::Float32(graph) => {
                Ok(eval_graph(py, graph, output_len, inputs)?.into_any())
            }
            PyGraphInner::Float64(graph) => {
                Ok(eval_graph(py, graph, output_len, inputs)?.into_any())
            }
        }
    }

    pub fn copy(&self) -> Self {
        self.clone()
    }

    pub fn __repr__(&self) -> String {
        let mut result = String::new();
        result.push_str("Graph(\n");
        match &self.inner {
            PyGraphInner::Float32(graph) => {
                for (i, node) in graph.as_ref().iter().enumerate() {
                    result.push_str(&format!("  Node {}: {:?}\n", i, node));
                }
            }
            PyGraphInner::Float64(graph) => {
                for (i, node) in graph.as_ref().iter().enumerate() {
                    result.push_str(&format!("  Node {}: {:?}\n", i, node));
                }
            }
        }

        result.push(')');
        result
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn __len__(&self) -> usize {
        match &self.inner {
            PyGraphInner::Float32(graph) => graph.as_ref().len(),
            PyGraphInner::Float64(graph) => graph.as_ref().len(),
        }
    }

    pub fn __eq__(&self, other: &PyGraph) -> bool {
        self.inner == other.inner
    }
}

impl From<Graph<Op<f32>>> for PyGraph {
    fn from(graph: Graph<Op<f32>>) -> Self {
        PyGraph {
            inner: PyGraphInner::Float32(graph.into()),
        }
    }
}

impl From<Graph<Op<f64>>> for PyGraph {
    fn from(graph: Graph<Op<f64>>) -> Self {
        PyGraph {
            inner: PyGraphInner::Float64(graph.into()),
        }
    }
}

impl Clone for PyGraph {
    fn clone(&self) -> Self {
        PyGraph {
            inner: self.inner.clone(),
        }
    }
}

impl IntoPyAnyObject for Graph<Op<f32>> {
    fn into_py<'py>(self, py: Python<'py>) -> RadiateResult<PyAnyObject> {
        let inner = graph_from_rust(py).call1(
            py,
            (PyGraph {
                inner: PyGraphInner::Float32(self.into()),
            }
            .into_bound_py_any(py)
            .unwrap(),),
        )?;

        Ok(PyAnyObject { inner })
    }
}

impl IntoPyAnyObject for Graph<Op<f64>> {
    fn into_py<'py>(self, py: Python<'py>) -> RadiateResult<PyAnyObject> {
        let inner = graph_from_rust(py).call1(
            py,
            (PyGraph {
                inner: PyGraphInner::Float64(self.into()),
            }
            .into_bound_py_any(py)
            .unwrap(),),
        )?;

        Ok(PyAnyObject { inner })
    }
}
