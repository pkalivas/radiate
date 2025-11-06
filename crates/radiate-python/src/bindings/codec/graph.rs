use crate::{PyGenotype, bindings::gp::PyGraph, object::Wrap};
use pyo3::{Bound, IntoPyObjectExt, Py, PyAny, PyResult, Python, pyclass, pymethods};
use radiate::{Codec, Genotype, GraphChromosome, GraphCodec, NodeType, Op};
use radiate_error::radiate_py_bail;
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
        let genotype: Genotype<GraphChromosome<Op<f32>>> = genotype.clone().into();
        let obj_value = self.codec.decode(&genotype);

        PyGraph {
            inner: obj_value,
            eval_cache: None,
        }
        .into_bound_py_any(py)
    }

    #[new]
    #[pyo3(signature = (graph_type=None, input_size=1, output_size=1, ops=None, max_nodes=None))]
    pub fn new<'py>(
        py: Python<'py>,
        graph_type: Option<&'py str>,
        input_size: usize,
        output_size: usize,
        ops: Option<HashMap<String, Vec<Py<PyAny>>>>,
        max_nodes: Option<usize>,
    ) -> PyResult<Self> {
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
                    radiate_py_bail!(
                        "Invalid node type key: {} - valid keys are: input, output, vertex, edge",
                    );
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
            Some("gru") => GraphCodec::gru(input_size, output_size, values),
            Some("lstm") => GraphCodec::lstm(input_size, output_size, values),
            _ => GraphCodec::directed(input_size, output_size, values),
        };

        Ok(Self {
            codec: match max_nodes {
                Some(max_nodes) => codec.with_max_nodes(max_nodes),
                None => codec,
            },
        })
    }
}
