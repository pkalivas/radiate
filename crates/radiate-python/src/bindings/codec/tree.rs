use crate::{PyGenotype, PyOp, bindings::gp::PyTree};
use pyo3::{Bound, IntoPyObjectExt, PyAny, PyResult, Python, pyclass, pymethods};
use radiate::{
    Codec, DataType, NodeType, Op, Tree, TreeChromosome, TreeCodec, dtype_names, ops::GpFloat,
};
use radiate_error::radiate_py_bail;
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
        // let genotype: radiate::Genotype<TreeChromosome<Op<f32>>> = genotype.clone().into();
        // let obj_value = self.codec.decode(&genotype);

        // PyTree { inner: obj_value }.into_bound_py_any(py)
        match &self.codec {
            PyTreeCodecInner::Float32(codec) => {
                PyTree::from(codec.decode(&genotype.clone().into()))
            }
            PyTreeCodecInner::Float64(codec) => {
                PyTree::from(codec.decode(&genotype.clone().into()))
            }
        }
        .into_bound_py_any(py)
    }

    #[new]
    #[pyo3(signature = (output_size=1, min_depth=1, max_size=30, ops=None, dtype=None))]
    pub fn new(
        output_size: usize,
        min_depth: usize,
        max_size: usize,
        ops: Option<HashMap<String, Vec<PyOp>>>,
        dtype: Option<&str>,
    ) -> PyResult<Self> {
        let datatype = crate::dtype_from_str(&dtype.unwrap_or_else(|| dtype_names::FLOAT32.into()));

        match datatype {
            DataType::Float32 => Ok(Self {
                codec: PyTreeCodecInner::Float32(build_typed_codec(
                    output_size,
                    min_depth,
                    max_size,
                    ops,
                )),
            }),
            DataType::Float64 => Ok(Self {
                codec: PyTreeCodecInner::Float64(build_typed_codec(
                    output_size,
                    min_depth,
                    max_size,
                    ops,
                )),
            }),
            _ => radiate_py_bail!("Unsupported data type for TreeCodec: {:datatype?}"),
        }
    }
}

unsafe impl Send for PyTreeCodec {}
unsafe impl Sync for PyTreeCodec {}

fn build_typed_codec<F: GpFloat>(
    output_size: usize,
    min_depth: usize,
    max_size: usize,
    ops: Option<HashMap<String, Vec<PyOp>>>,
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
                values.push((NodeType::Leaf, current_ops.clone()));
            } else if key == ROOT_NODE_TYPE {
                values.push((NodeType::Root, current_ops.clone()));
            } else if key == VERTEX_NODE_TYPE {
                values.push((NodeType::Vertex, current_ops.clone()));
            }
        }
    }

    TreeCodec::multi_root(min_depth, output_size, values)
        .constraint(move |node| node.size() <= max_size)
}
