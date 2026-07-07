use crate::object::Wrap;
use pyo3::exceptions::PyValueError;
use pyo3::{Borrowed, Py, PyErr, Python, intern, pyfunction, pymethods};
use pyo3::{FromPyObject, PyAny, PyResult, pyclass, types::PyAnyMethods};
use radiate::{
    Arity, DataType, Eval, Factory, Op,
    ops::{GpFloat, math_generic},
};
use radiate_error::radiate_py_bail;

fn _op_collection<F>(ops: Vec<Op<F>>) -> PyResult<Vec<PyOp>>
where
    PyOp: From<Op<F>>,
{
    Ok(ops.into_iter().map(PyOp::from).collect())
}

#[pyfunction]
pub fn _all_ops(dtype: &str) -> PyResult<Vec<PyOp>> {
    let datatype = crate::dtype_from_str(dtype);
    match datatype {
        DataType::Float32 => _op_collection(math_generic::all_ops::<f32>()),
        DataType::Float64 => _op_collection(math_generic::all_ops::<f64>()),
        _ => radiate_py_bail!("Unsupported data type for ops: {:datatype:?}"),
    }
}

#[pyfunction]
pub fn _activation_ops(dtype: &str) -> PyResult<Vec<PyOp>> {
    let datatype = crate::dtype_from_str(dtype);
    match datatype {
        DataType::Float32 => _op_collection(math_generic::activation_ops::<f32>()),
        DataType::Float64 => _op_collection(math_generic::activation_ops::<f64>()),
        _ => radiate_py_bail!("Unsupported data type for activation ops: {:datatype:?}"),
    }
}

#[pyfunction]
pub fn _edge_ops(dtype: &str) -> PyResult<Vec<PyOp>> {
    let datatype = crate::dtype_from_str(dtype);
    match datatype {
        DataType::Float32 => _op_collection(math_generic::edge_ops::<f32>()),
        DataType::Float64 => _op_collection(math_generic::edge_ops::<f64>()),
        _ => radiate_py_bail!("Unsupported data type for edge ops: {:datatype:?}"),
    }
}

/// `Op<T>` comes in two float widths (`f32`/`f64`) depending on which chromosome
/// produced it. A plain two-way enum is enough here — no need for a macro or a
/// trait object, the surface is tiny and both arms behave identically.
#[derive(Clone)]
pub enum PyOpInner {
    F32(Op<f32>),
    F64(Op<f64>),
}

impl From<PyOp> for Op<f32> {
    fn from(py_op: PyOp) -> Self {
        match py_op.0 {
            PyOpInner::F32(op) => op,
            PyOpInner::F64(_) => panic!("expected a 32-bit Op, found a 64-bit Op"),
        }
    }
}

impl From<PyOp> for Op<f64> {
    fn from(py_op: PyOp) -> Self {
        match py_op.0 {
            PyOpInner::F32(_) => panic!("expected a 64-bit Op, found a 32-bit Op"),
            PyOpInner::F64(op) => op,
        }
    }
}

impl From<Op<f32>> for PyOp {
    fn from(op: Op<f32>) -> Self {
        PyOp(PyOpInner::F32(op))
    }
}

impl From<Op<f64>> for PyOp {
    fn from(op: Op<f64>) -> Self {
        PyOp(PyOpInner::F64(op))
    }
}

#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct PyOp(pub PyOpInner);

#[pymethods]
impl PyOp {
    #[getter]
    pub fn name(&self) -> String {
        match &self.0 {
            PyOpInner::F32(op) => op.name().to_string(),
            PyOpInner::F64(op) => op.name().to_string(),
        }
    }

    #[getter]
    pub fn arity<'py>(&self, py: Python<'py>) -> PyResult<String> {
        let arity = match &self.0 {
            PyOpInner::F32(op) => op.arity(),
            PyOpInner::F64(op) => op.arity(),
        };
        match arity {
            Arity::Zero => Ok(intern!(py, "Zero").to_string()),
            Arity::Exact(n) => Ok(radiate_utils::intern!(format!("Exact({:?})", n)).to_string()),
            Arity::Any => Ok(intern!(py, "Any").to_string()),
        }
    }

    pub fn eval<'py>(&mut self, py: Python<'py>, inputs: Py<PyAny>) -> PyResult<f64> {
        let type_err = || {
            pyo3::exceptions::PyTypeError::new_err(format!(
                "Input must be either Vec[numeric] or a single numeric value but found: {:?}",
                inputs,
            ))
        };

        match &self.0 {
            PyOpInner::F32(op) => {
                let input_vec = inputs.extract::<Vec<f32>>(py).map_err(|_| type_err())?;
                Ok(op.eval(&input_vec) as f64)
            }
            PyOpInner::F64(op) => {
                let input_vec = inputs.extract::<Vec<f64>>(py).map_err(|_| type_err())?;
                Ok(op.eval(&input_vec))
            }
        }
    }

    pub fn new_instance(&self) -> PyResult<Self> {
        Ok(match &self.0 {
            PyOpInner::F32(op) => PyOp::from(op.new_instance(())),
            PyOpInner::F64(op) => PyOp::from(op.new_instance(())),
        })
    }
}

// impl PyOp {
//     /// The graph/tree codecs are still f32-only — used at the codec construction
//     /// boundary (`PyGraphCodec::new`/`PyTreeCodec::new`) to unwrap a `Vec<PyOp>`
//     /// supplied from Python, erroring clearly if a 64-bit Op sneaks in instead of
//     /// silently truncating or panicking on a bad downcast.
//     pub fn as_f32(&self) -> PyResult<Op<f32>> {
//         match &self.0 {
//             PyOpInner::F32(op) => Ok(op.clone()),
//             PyOpInner::F64(_) => Err(PyValueError::new_err(
//                 "expected a 32-bit Op, found a 64-bit Op (graph/tree codecs are f32-only for now)",
//             )),
//         }
//     }
// }

impl<'py> FromPyObject<'_, 'py> for Wrap<Vec<Op<f32>>> {
    type Error = PyErr;

    fn extract(ob: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
        let mut ops = Vec::new();
        for item in ob.try_iter()? {
            let wrap: Wrap<Op<f32>> = item?.extract()?;
            ops.push(wrap.0);
        }
        Ok(Wrap(ops))
    }
}

impl<'py> FromPyObject<'_, 'py> for Wrap<Op<f32>> {
    type Error = PyErr;

    fn extract(ob: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
        if ob.is_instance_of::<PyOp>() {
            let py_op: PyOp = ob.extract()?;
            return match py_op.0 {
                PyOpInner::F32(op) => Ok(Wrap(op)),
                PyOpInner::F64(_) => Err(PyValueError::new_err(
                    "expected a 32-bit Op, found a 64-bit Op",
                )),
            };
        }

        let name = ob.getattr("name")?.extract::<String>()?;
        let py_op = _create_op(ob.py(), name.as_str(), Some(ob.into()))?;
        match py_op.0 {
            PyOpInner::F32(op) => Ok(Wrap(op)),
            PyOpInner::F64(_) => unreachable!("_create_op only ever constructs 32-bit Ops"),
        }
    }
}

#[pyfunction]
#[pyo3(signature = (name, params=None))]
pub fn _create_op(py: Python<'_>, name: &str, params: Option<Py<PyAny>>) -> PyResult<PyOp> {
    let op = match name {
        "constant" => {
            let value: f32 = params
                .ok_or_else(|| PyValueError::new_err("Missing parameters for constant Op"))?
                .bind_borrowed(py)
                .get_item("value")?
                .extract()?;
            Op::constant(value)
        }
        "var" => {
            let index: usize = params
                .ok_or_else(|| PyValueError::new_err("Missing parameters for var Op"))?
                .bind_borrowed(py)
                .get_item("index")?
                .extract()?;
            Op::var(index)
        }
        "identity" => Op::identity(),
        "weight" => Op::weight(),
        "add" => Op::add(),
        "sub" => Op::sub(),
        "mul" => Op::mul(),
        "div" => Op::div(),
        "sum" => Op::sum(),
        "prod" => Op::prod(),
        "diff" => Op::diff(),
        "pow" => Op::pow(),
        "sqrt" => Op::sqrt(),
        "neg" => Op::neg(),
        "exp" => Op::exp(),
        "log" => Op::log(),
        "sin" => Op::sin(),
        "cos" => Op::cos(),
        "tan" => Op::tan(),
        "ceil" => Op::ceil(),
        "floor" => Op::floor(),
        "max" => Op::max(),
        "min" => Op::min(),
        "abs" => Op::abs(),
        "sigmoid" => Op::sigmoid(),
        "tanh" => Op::tanh(),
        "relu" => Op::relu(),
        "leaky_relu" => Op::leaky_relu(),
        "elu" => Op::elu(),
        "linear" => Op::linear(),
        "mish" => Op::mish(),
        "swish" => Op::swish(),
        "softplus" => Op::softplus(),
        _ => return Err(PyValueError::new_err(format!("Unknown Op name: {}", name))),
    };

    Ok(PyOp::from(op))
}
