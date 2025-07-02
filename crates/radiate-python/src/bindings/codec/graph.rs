use crate::{IntoPyObjectValue, ObjectValue, PyGenotype, object::Wrap};
use pyo3::{Bound, IntoPyObjectExt, Py, PyAny, PyResult, Python, pyclass, pymethods};
use radiate::{Codec, EvalMut, Graph, GraphChromosome, GraphCodec, GraphEvaluator, NodeType, Op};
use std::collections::HashMap;

const INPUT_NODE_TYPE: &str = "input";
const OUTPUT_NODE_TYPE: &str = "output";
const VERTEX_NODE_TYPE: &str = "vertex";
const EDGE_NODE_TYPE: &str = "edge";

#[pyclass]
#[derive(Clone)]
pub struct PyGraphCodec {
    pub codec: GraphCodec<Op<f32>>,
}

#[pymethods]
impl PyGraphCodec {
    pub fn encode_py(&self) -> PyGenotype {
        PyGenotype::from(self.codec.encode())
    }

    pub fn decode_py<'py>(
        &self,
        py: Python<'py>,
        genotype: &PyGenotype,
    ) -> PyResult<Bound<'py, PyAny>> {
        let genotype: radiate::Genotype<GraphChromosome<Op<f32>>> = genotype.clone().into();
        let obj_value = self.codec.decode(&genotype);

        PyGraph {
            inner: obj_value,
            eval_cache: None,
        }
        .into_bound_py_any(py)
    }

    #[new]
    #[pyo3(signature = (graph_type=None, input_size=1, output_size=1, ops=None))]
    pub fn new<'py>(
        py: Python<'py>,
        graph_type: Option<&'py str>,
        input_size: usize,
        output_size: usize,
        ops: Option<HashMap<String, Vec<Py<PyAny>>>>,
    ) -> Self {
        let mut values = Vec::new();
        if let Some(ops) = &ops {
            for (key, value) in ops.iter() {
                let ops_converted = value
                    .iter()
                    .map(|op| {
                        let wrap: Wrap<Op<f32>> = op.extract(py).unwrap();
                        wrap.0
                    })
                    .collect::<Vec<Op<f32>>>();

                if key == INPUT_NODE_TYPE {
                    values.push((NodeType::Input, ops_converted));
                } else if key == OUTPUT_NODE_TYPE {
                    values.push((NodeType::Output, ops_converted));
                } else if key == VERTEX_NODE_TYPE {
                    values.push((NodeType::Vertex, ops_converted));
                } else if key == EDGE_NODE_TYPE {
                    values.push((NodeType::Edge, ops_converted));
                } else {
                    panic!("Unknown node type: {}", key);
                }
            }
        }

        let codec = match graph_type {
            Some("recurrent") => GraphCodec::recurrent(input_size, output_size, values),
            Some("weighted_directed") => {
                GraphCodec::weighted_directed(input_size, output_size, values)
            }
            Some("weighted_recurrent") => {
                GraphCodec::weighted_recurrent(input_size, output_size, values)
            }
            _ => GraphCodec::directed(input_size, output_size, values),
        };

        Self { codec: codec }
    }
}

impl IntoPyObjectValue for Graph<Op<f32>> {
    fn into_py<'py>(self, py: Python<'py>) -> ObjectValue {
        ObjectValue {
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
#[derive(Clone)]
pub struct PyGraph {
    pub inner: Graph<Op<f32>>,
    pub eval_cache: Option<(Vec<f32>, Vec<f32>)>,
}

#[pymethods]
impl PyGraph {
    pub fn __repr__(&self) -> PyResult<String> {
        let mut result = String::new();
        result.push_str("Graph(\n");
        for (i, node) in self.inner.iter().enumerate() {
            result.push_str(&format!("  Node {}: {:?}\n", i, node));
        }
        result.push(')');
        Ok(result)
    }

    pub fn __str__(&self) -> PyResult<String> {
        let mut result = String::new();
        result.push_str("Graph(\n");
        for node in self.inner.iter() {
            result.push_str(&format!("{:?}\n", node));
        }
        result.push(')');
        Ok(result)
    }

    pub fn __len__(&self) -> PyResult<usize> {
        Ok(self.inner.len())
    }

    pub fn __eq__(&self, other: &PyGraph) -> PyResult<bool> {
        Ok(self.inner == other.inner)
    }

    pub fn eval(&mut self, inputs: Vec<Vec<f32>>) -> PyResult<Vec<Vec<f32>>> {
        if let Some(cache) = &self.eval_cache {
            let mut evaluator = GraphEvaluator::from((&self.inner, cache.clone()));
            let outputs = inputs
                .into_iter()
                .map(|input| evaluator.eval_mut(&input))
                .collect::<Vec<Vec<f32>>>();

            self.eval_cache = Some(evaluator.cache());
            return Ok(outputs);
        }

        let mut evaluator = GraphEvaluator::new(&self.inner);
        let outputs = inputs
            .into_iter()
            .map(|input| evaluator.eval_mut(&input))
            .collect::<Vec<Vec<f32>>>();
        self.eval_cache = Some(evaluator.cache());
        Ok(outputs)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.inner).unwrap()
    }
}
