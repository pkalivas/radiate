use crate::{IntoPyAnyObject, PyAnyObject};
use numpy::PyArrayDyn;
use pyo3::{Bound, IntoPyObjectExt, PyAny, PyResult, Python, pyclass, pymethods};
use radiate::{
    EvalMut, Graph, GraphEvaluator, GraphIterator, NodeType, Op, ToDot, graphs::GraphEvalCache,
};
use serde::{Deserialize, Serialize};

impl IntoPyAnyObject for Graph<Op<f32>> {
    fn into_py<'py>(self, py: Python<'py>) -> PyAnyObject {
        PyAnyObject {
            inner: PyGraph {
                inner: PyGraphInner::Float32(self, None),
            }
            .into_py_any(py)
            .unwrap(),
        }
    }
}

impl IntoPyAnyObject for Graph<Op<f64>> {
    fn into_py<'py>(self, py: Python<'py>) -> PyAnyObject {
        PyAnyObject {
            inner: PyGraph {
                inner: PyGraphInner::Float64(self, None),
            }
            .into_py_any(py)
            .unwrap(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub(crate) enum PyGraphInner {
    Float32(Graph<Op<f32>>, Option<GraphEvalCache<f32>>),
    Float64(Graph<Op<f64>>, Option<GraphEvalCache<f64>>),
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

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn to_dot(&self) -> String {
        match &self.inner {
            PyGraphInner::Float32(graph, _) => graph.to_dot(),
            PyGraphInner::Float64(graph, _) => graph.to_dot(),
        }
    }

    pub fn reset(&mut self) {
        match &mut self.inner {
            PyGraphInner::Float32(_, cache) => {
                *cache = None;
            }
            PyGraphInner::Float64(_, cache) => {
                *cache = None;
            }
        }
    }

    pub fn shape(&self) -> (usize, usize) {
        match &self.inner {
            PyGraphInner::Float32(graph, _) => (
                graph
                    .get_nodes_of_type(NodeType::Input)
                    .collect::<Vec<_>>()
                    .len(),
                graph
                    .get_nodes_of_type(NodeType::Output)
                    .collect::<Vec<_>>()
                    .len(),
            ),
            PyGraphInner::Float64(graph, _) => (
                graph
                    .get_nodes_of_type(NodeType::Input)
                    .collect::<Vec<_>>()
                    .len(),
                graph
                    .get_nodes_of_type(NodeType::Output)
                    .collect::<Vec<_>>()
                    .len(),
            ),
        }
    }

    pub fn eval_f32<'py>(
        &mut self,
        py: Python<'py>,
        inputs: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyArrayDyn<f32>>> {
        let shape = self.shape();
        if let PyGraphInner::Float32(graph, cache) = &mut self.inner {
            let mut evaluator = if cache.is_some() {
                let cache = cache.take().unwrap();
                GraphEvaluator::from((&graph, cache))
            } else {
                GraphEvaluator::new(graph)
            };

            let result =
                super::generic_eval_runner(py, shape.1, inputs, |slice| evaluator.eval_mut(slice));

            *cache = Some(evaluator.take_cache());
            result
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                "Graph is not of type f32",
            ))
        }
    }

    pub fn eval_f64<'py>(
        &mut self,
        py: Python<'py>,
        inputs: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyArrayDyn<f64>>> {
        let shape = self.shape();
        if let PyGraphInner::Float64(graph, cache) = &mut self.inner {
            let mut evaluator = if cache.is_some() {
                let cache = cache.take().unwrap();
                GraphEvaluator::from((&graph, cache))
            } else {
                GraphEvaluator::new(graph)
            };

            let result =
                super::generic_eval_runner(py, shape.1, inputs, |slice| evaluator.eval_mut(slice));

            *cache = Some(evaluator.take_cache());
            result
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                "Graph is not of type f64",
            ))
        }
    }

    pub fn copy(&self) -> Self {
        self.clone()
    }

    pub fn __repr__(&self) -> String {
        let mut result = String::new();
        result.push_str("Graph(\n");
        match &self.inner {
            PyGraphInner::Float32(graph, _) => {
                for (i, node) in graph.iter().enumerate() {
                    result.push_str(&format!("  Node {}: {:?}\n", i, node));
                }
            }
            PyGraphInner::Float64(graph, _) => {
                for (i, node) in graph.iter().enumerate() {
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
            PyGraphInner::Float32(graph, _) => graph.len(),
            PyGraphInner::Float64(graph, _) => graph.len(),
        }
    }

    pub fn __eq__(&self, other: &PyGraph) -> bool {
        self.inner == other.inner
    }
}

impl From<Graph<Op<f32>>> for PyGraph {
    fn from(graph: Graph<Op<f32>>) -> Self {
        PyGraph {
            inner: PyGraphInner::Float32(graph, None),
        }
    }
}

impl From<Graph<Op<f64>>> for PyGraph {
    fn from(graph: Graph<Op<f64>>) -> Self {
        PyGraph {
            inner: PyGraphInner::Float64(graph, None),
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
