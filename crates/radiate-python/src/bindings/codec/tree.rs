use crate::{PyGenotype, PyOp, bindings::gp::PyTree};
use pyo3::{Bound, IntoPyObjectExt, PyAny, PyResult, Python, pyclass, pymethods};
use radiate::{Codec, NodeType, Op, Tree, TreeChromosome, TreeCodec};
use std::collections::HashMap;

const LEAF_NODE_TYPE: &str = "leaf";
const ROOT_NODE_TYPE: &str = "root";
const VERTEX_NODE_TYPE: &str = "vertex";

#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct PyTreeCodec {
    pub codec: TreeCodec<Op<f32>, Vec<Tree<Op<f32>>>>,
}

#[pymethods]
impl PyTreeCodec {
    pub fn encode_py(&self) -> PyResult<PyGenotype> {
        Ok(PyGenotype::from(self.codec.encode()))
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
