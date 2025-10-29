use crate::object::Wrap;
use pyo3::exceptions::PyValueError;
use pyo3::{Borrowed, PyErr};
use pyo3::{FromPyObject, PyAny, PyResult, pyclass, types::PyAnyMethods};
use radiate::Op;

#[pyclass]
#[derive(Clone)]
pub struct PyOp {
    _inner: Op<f32>,
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
        }

        Err(PyValueError::new_err(format!("Unknown Op name: {}", name)))
    }
}
