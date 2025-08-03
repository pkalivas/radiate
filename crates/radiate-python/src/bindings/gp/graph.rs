use crate::{IntoPyAnyObject, PyAnyObject};
use pyo3::{IntoPyObjectExt, PyResult, Python, pyclass, pymethods};
use radiate::{EvalMut, Graph, GraphEvaluator, Op, ToDot, graphs::GraphEvalCache};
use serde::{Deserialize, Serialize};

impl IntoPyAnyObject for Graph<Op<f32>> {
    fn into_py<'py>(self, py: Python<'py>) -> PyAnyObject {
        PyAnyObject {
            inner: PyGraph {
                inner: self,
                eval_cache: None,
            }
            .into_py_any(py)
            .unwrap(),
        }
    }
}

#[pyclass]
#[derive(Serialize, Deserialize)]
pub struct PyGraph {
    pub inner: Graph<Op<f32>>,
    pub eval_cache: Option<GraphEvalCache<f32>>,
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
        self.inner.to_dot()
    }

    pub fn reset(&mut self) {
        self.eval_cache = None;
    }

    pub fn eval(&mut self, inputs: Vec<Vec<f32>>) -> PyResult<Vec<Vec<f32>>> {
        let mut evaluator = if self.eval_cache.is_some() {
            let cache = self.eval_cache.take().unwrap();
            GraphEvaluator::from((&self.inner, cache))
        } else {
            GraphEvaluator::new(&self.inner)
        };

        let outputs = inputs
            .into_iter()
            .map(|input| evaluator.eval_mut(&input))
            .collect::<Vec<Vec<f32>>>();

        self.eval_cache = Some(evaluator.take_cache());

        Ok(outputs)
    }

    pub fn __repr__(&self) -> String {
        let mut result = String::new();
        result.push_str("Graph(\n");
        for (i, node) in self.inner.iter().enumerate() {
            result.push_str(&format!("  Node {}: {:?}\n", i, node));
        }
        result.push(')');
        result
    }

    pub fn __str__(&self) -> String {
        let mut result = String::new();
        result.push_str("Graph(\n");
        for node in self.inner.iter() {
            result.push_str(&format!("{:?}\n", node));
        }
        result.push(')');

        result
    }

    pub fn __len__(&self) -> usize {
        self.inner.len()
    }

    pub fn __eq__(&self, other: &PyGraph) -> bool {
        self.inner == other.inner
    }
}

impl From<Graph<Op<f32>>> for PyGraph {
    fn from(graph: Graph<Op<f32>>) -> Self {
        PyGraph {
            inner: graph,
            eval_cache: None,
        }
    }
}
