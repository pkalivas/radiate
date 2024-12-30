use super::{Arity, Oper};

const MAX_VALUE: f32 = 1e+5_f32;
const MIN_VALUE: f32 = -1e+5_f32;
const ONE: f32 = 1.0_f32;
const ZERO: f32 = 0.0_f32;
const HALF: f32 = 0.5_f32;

fn clamp(value: f32) -> f32 {
    value.clamp(MIN_VALUE, MAX_VALUE)
}

fn activate<F>(vals: &[f32], f: F) -> f32
where
    F: Fn(f32) -> f32,
{
    let len = vals.len();
    if len == 0 {
        return ZERO;
    } else if len == 1 {
        return clamp(f(vals[0]));
    } else if len == 2 {
        return clamp(f(vals[0]) + f(vals[1]));
    } else if len == 3 {
        return clamp(f(vals[0]) + f(vals[1]) + f(vals[2]));
    }

    clamp(f(vals.iter().cloned().sum::<f32>()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Sin,
    Cos,
    Tan,
    Exp,
    Log,
    Abs,
    Neg,

    Sigmoid,
    ReLU,
    LeakyReLU,
    Tanh,
    Mish,
    Swish,
    Softplus,
    Linear,
}

impl Oper<f32> for MathOp {
    fn apply(&self, inputs: &[f32]) -> f32 {
        match self {
            MathOp::Add => activate(inputs, |x| x),
            MathOp::Sub => activate(inputs, |x| x),
            MathOp::Mul => activate(inputs, |x| x),
            MathOp::Div => activate(inputs, |x| x),
            MathOp::Pow => activate(inputs, |x| x),
            MathOp::Sin => activate(inputs, |x| x.sin()),
            MathOp::Cos => activate(inputs, |x| x.cos()),
            MathOp::Tan => activate(inputs, |x| x.tan()),
            MathOp::Exp => activate(inputs, |x| x.exp()),
            MathOp::Log => activate(inputs, |x| x.ln()),
            MathOp::Abs => activate(inputs, |x| x.abs()),
            MathOp::Neg => activate(inputs, |x| -x),

            MathOp::Sigmoid => activate(inputs, |x| ONE / (ONE + (-x).exp())),
            MathOp::ReLU => activate(inputs, |x| x.max(ZERO)),
            MathOp::LeakyReLU => activate(inputs, |x| x.max(ZERO) + HALF * x.min(ZERO)),
            MathOp::Tanh => activate(inputs, |x| x.tanh()),
            MathOp::Mish => activate(inputs, |x| x * (x.exp().ln_1p().tanh())),
            MathOp::Swish => activate(inputs, |x| x / (ONE + (-x).exp())),
            MathOp::Softplus => activate(inputs, |x| (ONE + x.exp()).ln()),
            MathOp::Linear => activate(inputs, |x| x),
        }
    }

    fn arity(&self) -> super::Arity {
        match self {
            MathOp::Add => Arity::Exact(2),
            MathOp::Sub => Arity::Exact(2),
            MathOp::Mul => Arity::Exact(2),
            MathOp::Div => Arity::Exact(2),
            MathOp::Pow => Arity::Exact(2),
            MathOp::Sin => Arity::Exact(1),
            MathOp::Cos => Arity::Exact(1),
            MathOp::Tan => Arity::Exact(1),
            MathOp::Exp => Arity::Exact(1),
            MathOp::Log => Arity::Exact(1),
            MathOp::Abs => Arity::Exact(1),
            MathOp::Neg => Arity::Exact(1),

            MathOp::Sigmoid => Arity::Any,
            MathOp::ReLU => Arity::Any,
            MathOp::LeakyReLU => Arity::Any,
            MathOp::Tanh => Arity::Any,
            MathOp::Mish => Arity::Any,
            MathOp::Swish => Arity::Any,
            MathOp::Softplus => Arity::Any,
            MathOp::Linear => Arity::Any,
        }
    }

    fn name(&self) -> &str {
        match self {
            MathOp::Add => "Add",
            MathOp::Sub => "Sub",
            MathOp::Mul => "Mul",
            MathOp::Div => "Div",
            MathOp::Pow => "Pow",
            MathOp::Sin => "Sin",
            MathOp::Cos => "Cos",
            MathOp::Tan => "Tan",
            MathOp::Exp => "Exp",
            MathOp::Log => "Log",
            MathOp::Abs => "Abs",
            MathOp::Neg => "Neg",

            MathOp::Sigmoid => "Sigmoid",
            MathOp::ReLU => "ReLU",
            MathOp::LeakyReLU => "LeakyReLU",
            MathOp::Tanh => "Tanh",
            MathOp::Mish => "Mish",
            MathOp::Swish => "Swish",
            MathOp::Softplus => "Softplus",
            MathOp::Linear => "Linear",
        }
    }
}

pub struct WeightOp {
    pub weight: f32,
    pub min: f32,
    pub max: f32,
}

impl Oper<f32> for WeightOp {
    fn apply(&self, inputs: &[f32]) -> f32 {
        if inputs.len() != *self.arity() {
            panic!("Invalid number of inputs for weight operation");
        }

        clamp(inputs[0] + self.weight)
    }

    fn arity(&self) -> super::Arity {
        Arity::Exact(1)
    }

    fn name(&self) -> &str {
        "w"
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(1.0), 1.0);
        assert_eq!(clamp(-1.0), -1.0);
        assert_eq!(clamp(1e+6), MAX_VALUE);
        assert_eq!(clamp(-1e+6), MIN_VALUE);
        assert_eq!(clamp(f32::NAN), MIN_VALUE);
        assert_eq!(clamp(f32::INFINITY), MAX_VALUE);
        assert_eq!(clamp(f32::NEG_INFINITY), MIN_VALUE);
        assert_eq!(clamp(f32::EPSILON), f32::EPSILON);
    }
}
