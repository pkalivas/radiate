use pyo3::{Py, Python, intern, pyfunction, pymethods};
use pyo3::{PyAny, PyResult, pyclass, types::PyAnyMethods};
use pyo3::{exceptions::PyValueError, prelude::FromPyObjectOwned};
use radiate::{
    Arity, DataType, Eval, Factory, Op,
    ops::{OpFloat, math},
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
        DataType::Float32 => _op_collection(math::all_ops::<f32>()),
        DataType::Float64 => _op_collection(math::all_ops::<f64>()),
        _ => radiate_py_bail!("Unsupported data type for ops: {:datatype:?}"),
    }
}

#[pyfunction]
pub fn _activation_ops(dtype: &str) -> PyResult<Vec<PyOp>> {
    let datatype = crate::dtype_from_str(dtype);
    match datatype {
        DataType::Float32 => _op_collection(math::activation_ops::<f32>()),
        DataType::Float64 => _op_collection(math::activation_ops::<f64>()),
        _ => radiate_py_bail!("Unsupported data type for activation ops: {:datatype:?}"),
    }
}

#[pyfunction]
pub fn _edge_ops(dtype: &str) -> PyResult<Vec<PyOp>> {
    let datatype = crate::dtype_from_str(dtype);
    match datatype {
        DataType::Float32 => _op_collection(math::edge_ops::<f32>()),
        DataType::Float64 => _op_collection(math::edge_ops::<f64>()),
        _ => radiate_py_bail!("Unsupported data type for edge ops: {:datatype:?}"),
    }
}

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

#[pyfunction]
#[pyo3(signature = (name, dtype, params=None))]
pub fn _create_op<'py>(
    py: Python<'py>,
    name: &str,
    dtype: &str,
    params: Option<Py<PyAny>>,
) -> PyResult<PyOp> {
    let datatype = crate::dtype_from_str(dtype);
    match datatype {
        DataType::Float32 => build_typed_op::<f32>(py, name, params),
        DataType::Float64 => build_typed_op::<f64>(py, name, params),
        _ => radiate_py_bail!("Unsupported data type for op creation: {:datatype:?}"),
    }
}

fn build_typed_op<'py, F>(py: Python<'py>, name: &str, params: Option<Py<PyAny>>) -> PyResult<PyOp>
where
    PyOp: From<Op<F>>,
    F: FromPyObjectOwned<'py> + OpFloat,
{
    let op = match name {
        "constant" => {
            let value = params
                .ok_or_else(|| PyValueError::new_err("Missing parameters for constant Op"))?
                .bind(py)
                .get_item("value")?
                .extract::<F>()
                .map_err(|_| PyValueError::new_err("Invalid value for constant Op"))?;
            Op::constant(value)
        }
        "var" => {
            let index = params
                .ok_or_else(|| PyValueError::new_err("Missing parameters for var Op"))?
                .bind_borrowed(py)
                .get_item("index")?
                .extract::<usize>()?;
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
        _ => radiate_py_bail!(format!("Unknown Op name: {:?}", name)),
    };

    Ok(PyOp::from(op))
}
