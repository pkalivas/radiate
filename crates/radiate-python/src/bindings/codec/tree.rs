use crate::{PyGenotype, PyOp, bindings::gp::PyTree};
use pyo3::{Bound, IntoPyObjectExt, PyAny, PyResult, Python, pyclass, pymethods};
use radiate::{Codec, NodeType, Op, Tree, TreeChromosome, TreeCodec, ops::GpFloat};
use std::collections::HashMap;

const LEAF_NODE_TYPE: &str = "leaf";
const ROOT_NODE_TYPE: &str = "root";
const VERTEX_NODE_TYPE: &str = "vertex";

#[derive(Clone)]
pub enum PyTreeCodecInner {
    Float32(TreeCodec<Op<f32>, Vec<Tree<Op<f32>>>>),
    Float64(TreeCodec<Op<f64>, Vec<Tree<Op<f64>>>>),
}

#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct PyTreeCodec {
    pub codec: PyTreeCodecInner,
}

#[pymethods]
impl PyTreeCodec {
    pub fn encode_py(&self) -> PyResult<PyGenotype> {
        match &self.codec {
            PyTreeCodecInner::Float32(codec) => Ok(PyGenotype::from(codec.encode())),
            PyTreeCodecInner::Float64(codec) => Ok(PyGenotype::from(codec.encode())),
        }
    }

    pub fn decode_py<'py>(
        &self,
        py: Python<'py>,
        genotype: &PyGenotype,
    ) -> PyResult<Bound<'py, PyAny>> {
        let genotype: radiate::Genotype<TreeChromosome<Op<f32>>> = genotype.clone().into();
        let obj_value = self.codec.decode(&genotype);

        PyTree { inner: obj_value }.into_bound_py_any(py)
    }

    #[new]
    #[pyo3(signature = (output_size=1, min_depth=1, max_size=30, ops=None))]
    pub fn new(
        output_size: usize,
        min_depth: usize,
        max_size: usize,
        ops: Option<HashMap<String, Vec<PyOp>>>,
    ) -> Self {
        let mut values = Vec::new();
        if let Some(ops) = &ops {
            for (key, value) in ops.iter() {
                if key == LEAF_NODE_TYPE {
                    values.push((
                        NodeType::Leaf,
                        value.iter().map(|op| op.0.clone()).collect(),
                    ));
                } else if key == ROOT_NODE_TYPE {
                    values.push((
                        NodeType::Root,
                        value.iter().map(|op| op.0.clone()).collect(),
                    ));
                } else if key == VERTEX_NODE_TYPE {
                    values.push((
                        NodeType::Vertex,
                        value.iter().map(|op| op.0.clone()).collect(),
                    ));
                }
            }
        }

        Self {
            codec: TreeCodec::multi_root(min_depth, output_size, values)
                .constraint(move |node| node.size() <= max_size),
        }
    }
}

unsafe impl Send for PyTreeCodec {}
unsafe impl Sync for PyTreeCodec {}

fn build_typed_codec<F: GpFloat>(
    graph_type: Option<&str>,
    input_size: usize,
    output_size: usize,
    ops: Option<HashMap<String, Vec<PyOp>>>,
    max_nodes: Option<usize>,
) -> TreeCodec<Op<F>>
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
            if key == LEAF_NODE_TYPE {
                values.push((NodeType::Leaf, current_ops));
            } else if key == ROOT_NODE_TYPE {
                values.push((NodeType::Root, current_ops));
            } else if key == VERTEX_NODE_TYPE {
                values.push((NodeType::Vertex, current_ops));
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
