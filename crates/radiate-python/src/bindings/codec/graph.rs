use crate::{PyCodec, PyGenotype, conversion::Wrap};
use pyo3::{
    Bound, FromPyObject, Py, PyAny, PyResult, Python, pyclass, pymethods, types::PyAnyMethods,
};
use radiate::{
    Chromosome, Codec, Graph, GraphChromosome, GraphCodec, GraphNode, NodeStore, NodeType, Op,
};
use std::collections::HashMap;

const INPUT_NODE_TYPE: &str = "input";
const OUTPUT_NODE_TYPE: &str = "output";
const VERTEX_NODE_TYPE: &str = "vertex";
const EDGE_NODE_TYPE: &str = "edge";

#[pyclass]
#[derive(Clone)]
pub struct PyGraphCodec {
    pub codec: PyCodec<GraphChromosome<Op<f32>>, Graph<Op<f32>>>,
}

#[pymethods]
impl PyGraphCodec {
    pub fn encode_py(&self) -> PyGenotype {
        PyGenotype::from(self.codec.encode())
    }

    #[staticmethod]
    #[pyo3(signature = (input_size=10, output_size=20, ops=None))]
    pub fn directed<'py>(
        py: Python<'py>,
        input_size: usize,
        output_size: usize,
        ops: Option<HashMap<String, Vec<Py<PyAny>>>>,
    ) -> Self {
        let store = NodeStore::new();
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
                    store.insert(NodeType::Input, ops_converted);
                } else if key == OUTPUT_NODE_TYPE {
                    store.insert(NodeType::Output, ops_converted);
                } else if key == VERTEX_NODE_TYPE {
                    store.insert(NodeType::Vertex, ops_converted);
                } else if key == EDGE_NODE_TYPE {
                    store.insert(NodeType::Edge, ops_converted);
                } else {
                    panic!("Unknown node type: {}", key);
                }
            }
        }

        let codec = GraphCodec::directed(input_size, output_size, store);
        PyGraphCodec {
            codec: PyCodec::new()
                .with_encoder(move || codec.encode())
                .with_decoder(|_, genotype| {
                    let graph = Graph::new(
                        genotype
                            .iter()
                            .flat_map(|chrom| chrom.iter())
                            .cloned()
                            .collect::<Vec<GraphNode<Op<f32>>>>(),
                    );
                    graph
                }),
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyGraph {
    pub inner: Graph<Op<f32>>,
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
        Ok(format!("Graph with {} nodes", self.inner.len()))
    }
}

impl<'py> FromPyObject<'py> for Wrap<Vec<Op<f32>>> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let mut ops = Vec::new();
        for item in ob.try_iter()? {
            let wrap: Wrap<Op<f32>> = item?.extract()?;
            ops.push(wrap.0);
        }
        Ok(Wrap(ops))
    }
}

impl<'py> FromPyObject<'py> for Wrap<Op<f32>> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let name: String = ob.getattr("name")?.extract()?;

        if name == "constant" {
            let value: f32 = ob.get_item("value")?.extract()?;
            return Ok(Wrap(Op::constant(value)));
        } else if name == "var" {
            let index: usize = ob.get_item("index")?.extract()?;
            return Ok(Wrap(Op::var(index)));
        } else if name == "identity" {
            return Ok(Wrap(Op::identity()));
        } else if name == "weight" {
            return Ok(Wrap(Op::weight()));
        } else if name == "add" {
            return Ok(Wrap(Op::add()));
        } else if name == "sub" {
            return Ok(Wrap(Op::sub()));
        } else if name == "mul" {
            return Ok(Wrap(Op::mul()));
        } else if name == "div" {
            return Ok(Wrap(Op::div()));
        } else if name == "sum" {
            return Ok(Wrap(Op::sum()));
        } else if name == "prod" {
            return Ok(Wrap(Op::prod()));
        } else if name == "diff" {
            return Ok(Wrap(Op::diff()));
        } else if name == "pow" {
            return Ok(Wrap(Op::pow()));
        } else if name == "sqrt" {
            return Ok(Wrap(Op::sqrt()));
        } else if name == "neg" {
            return Ok(Wrap(Op::neg()));
        } else if name == "exp" {
            return Ok(Wrap(Op::exp()));
        } else if name == "log" {
            return Ok(Wrap(Op::log()));
        } else if name == "sin" {
            return Ok(Wrap(Op::sin()));
        } else if name == "cos" {
            return Ok(Wrap(Op::cos()));
        } else if name == "tan" {
            return Ok(Wrap(Op::tan()));
        } else if name == "ceil" {
            return Ok(Wrap(Op::ceil()));
        } else if name == "floor" {
            return Ok(Wrap(Op::floor()));
        } else if name == "max" {
            return Ok(Wrap(Op::max()));
        } else if name == "min" {
            return Ok(Wrap(Op::min()));
        } else if name == "abs" {
            return Ok(Wrap(Op::abs()));
        } else if name == "sigmoid" {
            return Ok(Wrap(Op::sigmoid()));
        } else if name == "tanh" {
            return Ok(Wrap(Op::tanh()));
        } else if name == "relu" {
            return Ok(Wrap(Op::relu()));
        } else if name == "leaky_relu" {
            return Ok(Wrap(Op::leaky_relu()));
        } else if name == "elu" {
            return Ok(Wrap(Op::elu()));
        } else if name == "linear" {
            return Ok(Wrap(Op::linear()));
        } else if name == "mish" {
            return Ok(Wrap(Op::mish()));
        } else if name == "swish" {
            return Ok(Wrap(Op::swish()));
        } else if name == "softplus" {
            return Ok(Wrap(Op::softplus()));
        } else if name == "softmax" {
            return Ok(Wrap(Op::softmax()));
        }

        Ok(Wrap(Op::abs()))
    }
}
