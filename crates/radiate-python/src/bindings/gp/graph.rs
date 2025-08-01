use crate::{IntoPyAnyObject, PyAnyObject};
use pyo3::{IntoPyObjectExt, PyResult, Python, pyclass, pymethods};
use radiate::{
    EvalMut, Graph, GraphEvaluator, GraphIterator, GraphNode, Node, NodeType, Op,
    graphs::GraphEvalCache,
};
use std::collections::BTreeSet;

const INPUT_NODE_TYPE: &str = "input";
const OUTPUT_NODE_TYPE: &str = "output";
const VERTEX_NODE_TYPE: &str = "vertex";
const EDGE_NODE_TYPE: &str = "edge";

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
pub struct PyGraph {
    pub inner: Graph<Op<f32>>,
    pub eval_cache: Option<GraphEvalCache<f32>>,
}

#[pymethods]
impl PyGraph {
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

    pub fn nodes(&self) -> Vec<PyGraphNode> {
        self.inner
            .iter_topological()
            .map(|node| PyGraphNode {
                inner: node.clone(),
            })
            .collect()
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

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.inner).unwrap()
    }
}

#[pyclass]
pub struct PyGraphNode {
    pub inner: GraphNode<Op<f32>>,
}

#[pymethods]
impl PyGraphNode {
    pub fn __repr__(&self) -> String {
        format!("{:?}", self.inner)
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn id(&self) -> u64 {
        *(*self.inner.id())
    }

    pub fn index(&self) -> usize {
        self.inner.index()
    }

    pub fn arity(&self) -> usize {
        *self.inner.arity()
    }

    pub fn is_recurrent(&self) -> bool {
        self.inner.is_recurrent()
    }

    pub fn incoming(&self) -> &BTreeSet<usize> {
        self.inner.incoming()
    }

    pub fn outgoing(&self) -> &BTreeSet<usize> {
        self.inner.outgoing()
    }

    pub fn node_type(&self) -> &str {
        match self.inner.node_type() {
            NodeType::Input => INPUT_NODE_TYPE,
            NodeType::Output => OUTPUT_NODE_TYPE,
            NodeType::Vertex => VERTEX_NODE_TYPE,
            NodeType::Edge => EDGE_NODE_TYPE,
            _ => "unknown",
        }
    }

    pub fn value(&self) -> String {
        self.inner.value().to_string()
    }
}
