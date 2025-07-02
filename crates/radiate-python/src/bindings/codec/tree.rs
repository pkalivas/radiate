use crate::{IntoPyObjectValue, ObjectValue, PyGenotype, object::Wrap};
use pyo3::{Bound, IntoPyObjectExt, Py, PyAny, PyResult, Python, pyclass, pymethods};
use radiate::{Codec, Eval, Format, NodeType, Op, Tree, TreeChromosome, TreeCodec};
use std::collections::HashMap;

const LEAF_NODE_TYPE: &str = "leaf";
const ROOT_NODE_TYPE: &str = "root";
const VERTEX_NODE_TYPE: &str = "vertex";

#[pyclass(unsendable)]
#[derive(Clone)]
pub struct PyTreeCodec {
    pub codec: TreeCodec<Op<f32>, Vec<Tree<Op<f32>>>>,
}

#[pymethods]
impl PyTreeCodec {
    pub fn encode_py(&self) -> PyGenotype {
        PyGenotype::from(self.codec.encode())
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
    pub fn new<'py>(
        py: Python<'py>,
        output_size: usize,
        min_depth: usize,
        max_size: usize,
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

                if key == LEAF_NODE_TYPE {
                    values.push((NodeType::Leaf, ops_converted));
                } else if key == ROOT_NODE_TYPE {
                    values.push((NodeType::Root, ops_converted));
                } else if key == VERTEX_NODE_TYPE {
                    values.push((NodeType::Vertex, ops_converted));
                } else {
                    panic!("Unknown node type: {}", key);
                }
            }
        }

        Self {
            codec: TreeCodec::multi_root(min_depth, output_size, values)
                .constraint(move |node| node.size() <= max_size),
        }
    }
}

impl IntoPyObjectValue for Vec<Tree<Op<f32>>> {
    fn into_py<'py>(self, py: Python<'py>) -> ObjectValue {
        ObjectValue {
            inner: PyTree { inner: self }.into_py_any(py).unwrap(),
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyTree {
    pub inner: Vec<Tree<Op<f32>>>,
}

#[pymethods]
impl PyTree {
    pub fn __repr__(&self) -> PyResult<String> {
        let mut result = String::new();
        result.push_str("Tree(\n");
        for tree in self.inner.iter() {
            result.push_str(tree.format().as_str());
        }
        result.push(')');
        Ok(result)
    }

    pub fn __str__(&self) -> PyResult<String> {
        let mut result = String::new();
        result.push_str("Tree(\n");
        for node in self.inner.iter() {
            result.push_str(&format!("{:?}\n", node));
        }
        result.push(')');
        Ok(result)
    }

    pub fn __len__(&self) -> PyResult<usize> {
        Ok(self.inner.len())
    }

    pub fn __eq__(&self, other: &PyTree) -> PyResult<bool> {
        Ok(self.inner == other.inner)
    }

    pub fn eval(&mut self, inputs: Vec<Vec<f32>>) -> PyResult<Vec<Vec<f32>>> {
        println!("Evaluating tree with inputs: {:?}", inputs);
        Ok(inputs
            .into_iter()
            .map(|input| self.inner.eval(&input))
            .collect::<Vec<Vec<f32>>>())
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.inner).unwrap()
    }
}
