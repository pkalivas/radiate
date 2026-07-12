use crate::{PyGenotype, PyOp, Wrap, bindings::gp::PyGraph};
use pyo3::{Bound, IntoPyObjectExt, PyAny, PyResult, Python, pyclass, pymethods};
use radiate::{Codec, DataType, GraphCodec, NodeType, Op, dtype_names, ops::OpFloat};
use radiate_error::radiate_py_bail;
use std::collections::HashMap;

const INPUT_NODE_TYPE: &str = "input";
const OUTPUT_NODE_TYPE: &str = "output";
const VERTEX_NODE_TYPE: &str = "vertex";
const EDGE_NODE_TYPE: &str = "edge";

#[derive(Clone)]
pub enum PyGraphCodecInner {
    Float32(GraphCodec<Op<f32>>),
    Float64(GraphCodec<Op<f64>>),
}

#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct PyGraphCodec {
    pub codec: PyGraphCodecInner,
}

#[pymethods]
impl PyGraphCodec {
    #[new]
    #[pyo3(signature = (graph_type=None, input_size=1, output_size=1, ops=None, max_nodes=None, dtype=None))]
    pub fn new(
        graph_type: Option<&str>,
        input_size: usize,
        output_size: usize,
        ops: Option<HashMap<String, Vec<PyOp>>>,
        max_nodes: Option<usize>,
        dtype: Option<&str>,
    ) -> PyResult<Self> {
        let datatype = crate::dtype_from_str(dtype.unwrap_or(dtype_names::FLOAT64));

        match datatype {
            DataType::Float32 => Ok(PyGraphCodec {
                codec: PyGraphCodecInner::Float32(build_typed_codec::<f32>(
                    graph_type,
                    input_size,
                    output_size,
                    ops,
                    max_nodes,
                )),
            }),
            DataType::Float64 => Ok(PyGraphCodec {
                codec: PyGraphCodecInner::Float64(build_typed_codec::<f64>(
                    graph_type,
                    input_size,
                    output_size,
                    ops,
                    max_nodes,
                )),
            }),
            _ => radiate_py_bail!("Unsupported data type for graph codec: {:datatype:?}"),
        }
    }

    pub fn encode_py(&self) -> PyResult<PyGenotype> {
        match &self.codec {
            PyGraphCodecInner::Float32(codec) => Ok(PyGenotype::from(codec.encode())),
            PyGraphCodecInner::Float64(codec) => Ok(PyGenotype::from(codec.encode())),
        }
    }

    pub fn decode_py<'py>(
        &self,
        py: Python<'py>,
        genotype: &PyGenotype,
    ) -> PyResult<Bound<'py, PyAny>> {
        let obj_value = match &self.codec {
            PyGraphCodecInner::Float32(codec) => {
                PyGraph::from(codec.decode(&genotype.clone().into()))
            }
            PyGraphCodecInner::Float64(codec) => {
                PyGraph::from(codec.decode(&genotype.clone().into()))
            }
        };

        obj_value.into_bound_py_any(py)
    }

    pub fn dtype<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let dtype = match &self.codec {
            PyGraphCodecInner::Float32(_) => DataType::Float32,
            PyGraphCodecInner::Float64(_) => DataType::Float64,
        };

        Wrap(dtype).into_bound_py_any(py)
    }
}

fn build_typed_codec<F: OpFloat>(
    graph_type: Option<&str>,
    input_size: usize,
    output_size: usize,
    ops: Option<HashMap<String, Vec<PyOp>>>,
    max_nodes: Option<usize>,
) -> GraphCodec<Op<F>>
where
    PyOp: Into<Op<F>> + Clone,
{
    let mut values: Vec<(NodeType, Vec<Op<F>>)> = Vec::new();
    if let Some(ops) = &ops {
        for (key, value) in ops.iter() {
            let current_ops = value
                .iter()
                .map(|op| op.clone().into())
                .collect::<Vec<Op<F>>>();

            if key == INPUT_NODE_TYPE {
                values.push((NodeType::Input, current_ops));
            } else if key == OUTPUT_NODE_TYPE {
                values.push((NodeType::Output, current_ops));
            } else if key == VERTEX_NODE_TYPE {
                values.push((NodeType::Vertex, current_ops));
            } else if key == EDGE_NODE_TYPE {
                values.push((NodeType::Edge, current_ops));
            }
        }
    }

    let codec = match graph_type {
        Some("recurrent") => GraphCodec::recurrent(input_size, output_size, values),
        Some("weighted_directed") => GraphCodec::weighted_directed(input_size, output_size, values),
        Some("weighted_recurrent") => {
            GraphCodec::weighted_recurrent(input_size, output_size, values)
        }
        Some("gru") => GraphCodec::gru(input_size, output_size, values),
        Some("lstm") => GraphCodec::lstm(input_size, output_size, values),
        _ => GraphCodec::directed(input_size, output_size, values),
    };

    match max_nodes {
        Some(max_nodes) => codec.with_max_nodes(max_nodes),
        None => codec,
    }
}
