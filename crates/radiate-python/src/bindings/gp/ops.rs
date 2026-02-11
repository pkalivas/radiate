use crate::object::Wrap;
use pyo3::exceptions::PyValueError;
use pyo3::{Borrowed, Py, PyErr, Python, intern, pyfunction, pymethods};
use pyo3::{FromPyObject, PyAny, PyResult, pyclass, types::PyAnyMethods};
use radiate::{Arity, Eval, Factory, Op};

#[pyfunction]
pub fn _all_ops() -> Vec<PyOp> {
    vec![
        Op::constant(0.0),
        Op::var(0),
        Op::identity(),
        Op::weight(),
        Op::add(),
        Op::sub(),
        Op::mul(),
        Op::div(),
        Op::sum(),
        Op::prod(),
        Op::diff(),
        Op::pow(),
        Op::sqrt(),
        Op::neg(),
        Op::exp(),
        Op::log(),
        Op::sin(),
        Op::cos(),
        Op::tan(),
        Op::ceil(),
        Op::floor(),
        Op::max(),
        Op::min(),
        Op::abs(),
        Op::sigmoid(),
        Op::tanh(),
        Op::relu(),
        Op::leaky_relu(),
        Op::elu(),
        Op::linear(),
        Op::mish(),
        Op::swish(),
        Op::softplus(),
    ]
    .into_iter()
    .map(|op| PyOp(op))
    .collect()
}

#[pyfunction]
pub fn _activation_ops() -> Vec<PyOp> {
    vec![
        Op::sigmoid(),
        Op::tanh(),
        Op::relu(),
        Op::leaky_relu(),
        Op::elu(),
        Op::linear(),
        Op::mish(),
        Op::swish(),
        Op::softplus(),
    ]
    .into_iter()
    .map(|op| PyOp(op))
    .collect()
}

#[pyfunction]
pub fn _edge_ops() -> Vec<PyOp> {
    vec![Op::weight(), Op::identity()]
        .into_iter()
        .map(|op| PyOp(op))
        .collect()
}

#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct PyOp(pub Op<f32>);

#[pymethods]
impl PyOp {
    #[getter]
    pub fn name(&self) -> String {
        self.0.name().to_string()
    }

    #[getter]
    pub fn arity<'py>(&self, py: Python<'py>) -> PyResult<String> {
        match self.0.arity() {
            Arity::Zero => Ok(intern!(py, "Zero").to_string()),
            Arity::Exact(n) => Ok(radiate_utils::intern!(format!("Exact({:?})", n)).to_string()),
            Arity::Any => Ok(intern!(py, "Any").to_string()),
        }
    }

    pub fn eval<'py>(&mut self, py: Python<'py>, inputs: Py<PyAny>) -> PyResult<f32> {
        if let Ok(input_vec) = inputs.extract::<Vec<f32>>(py) {
            let output = self.0.eval(&input_vec);
            Ok(output)
        } else {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "Input must be either Vec[float] or Vec[Vec[float]]",
            ));
        }
    }

    pub fn new_instance(&self) -> PyResult<Self> {
        Ok(PyOp(self.0.new_instance(())))
    }
}

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
            return Ok(Wrap(py_op.0));
        }

        let name = ob.getattr("name")?.extract::<String>()?;
        let py_op = _create_op(ob.py(), name.as_str(), Some(ob.clone().into()))?;
        Ok(Wrap(py_op.0))
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

    Ok(PyOp(op))
}
