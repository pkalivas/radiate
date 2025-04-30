use super::Op;
use crate::Arity;
use radiate_core::random_provider;
use std::sync::Arc;

const MAX_VALUE: f32 = 1e+10_f32;
const MIN_VALUE: f32 = -1e+10_f32;
const ONE: f32 = 1.0_f32;
const ZERO: f32 = 0.0_f32;
const TWO: f32 = 2.0_f32;
const HALF: f32 = 0.5_f32;
const TENTH: f32 = 0.1_f32;

/// Clamp a value to the range [-1e+10, 1e+10]. Without this, values can quickly become
/// too large or too small to be useful.
fn clamp(value: f32) -> f32 {
    if value.is_nan() {
        return ZERO;
    }

    value.clamp(MIN_VALUE, MAX_VALUE)
}

/// Aggregate a slice of 'f32' values by summing them, then applying a function to the result.
/// There usually arent too many inputs, so we can use an if statement to handle a few of the
/// common cases - vals with a len <= 5.
fn aggregate<F>(vals: &[f32], f: F) -> f32
where
    F: Fn(f32) -> f32,
{
    let len = vals.len();
    if len == 0 {
        return ZERO;
    } else if len == 1 {
        return vals[0];
    } else if len == 2 {
        return f(vals[0] + vals[1]);
    } else if len == 3 {
        return f(vals[0] + vals[1] + vals[2]);
    } else if len == 4 {
        return f(vals[0] + vals[1] + vals[2] + vals[3]);
    } else if len == 5 {
        return f(vals[0] + vals[1] + vals[2] + vals[3] + vals[4]);
    }

    f(vals.iter().cloned().sum::<f32>())
}

pub enum MathOperation {
    Add,
    Sub,
    Mul,
    Div,
    Sum,
    Prod,
    Diff,
    Neg,
    Pow,
    Sqrt,
    Abs,
    Exp,
    Log,
    Sin,
    Cos,
    Tan,
    Ceil,
    Floor,
    Max,
    Min,
}

/// Implementations of the `MathOperation` enum. These are the basic math operations.
/// Each operation takes a slice of `f32` values and returns a single `f32` value.
impl MathOperation {
    pub fn apply(&self, inputs: &[f32]) -> f32 {
        match self {
            MathOperation::Add => clamp(inputs[0] + inputs[1]),
            MathOperation::Sub => clamp(inputs[0] - inputs[1]),
            MathOperation::Mul => clamp(inputs[0] * inputs[1]),
            MathOperation::Div => {
                if inputs[1].abs() < MIN_VALUE {
                    clamp(inputs[0] / ONE)
                } else {
                    clamp(inputs[0] / inputs[1])
                }
            }
            MathOperation::Sum => clamp(aggregate(inputs, |x| x)),
            MathOperation::Diff => clamp(inputs.iter().cloned().fold(ZERO, |acc, x| acc - x)),
            MathOperation::Prod => clamp(inputs.iter().product()),
            MathOperation::Neg => clamp(-inputs[0]),
            MathOperation::Pow => clamp(inputs[0].powf(inputs[1])),
            MathOperation::Sqrt => clamp(inputs[0].sqrt()),
            MathOperation::Abs => clamp(inputs[0].abs()),
            MathOperation::Exp => clamp(inputs[0].exp()),
            MathOperation::Log => clamp(inputs[0].ln()),
            MathOperation::Sin => clamp(inputs[0].sin()),
            MathOperation::Cos => clamp(inputs[0].cos()),
            MathOperation::Tan => clamp(inputs[0].tan()),
            MathOperation::Ceil => clamp(inputs[0].ceil()),
            MathOperation::Floor => clamp(inputs[0].floor()),
            MathOperation::Max => clamp(inputs.iter().cloned().fold(MIN_VALUE, f32::max)),
            MathOperation::Min => clamp(inputs.iter().cloned().fold(MAX_VALUE, f32::min)),
        }
    }
}

pub enum ActivationOperation {
    Sigmoid,
    Tanh,
    ReLU,
    LeakyReLU,
    ELU,
    Linear,
    Mish,
    Swish,
    Softplus,
    Softmax,
}

/// Implementations of the `ActivationOperation` enum. These are the basic activation functions used
/// in neural networks. However, they are particularly useful in this context because they can
/// accept any number of inputs. Thus, they act as reducers or aggregates and are a key part of
/// being able to define complex 'Graph' and 'Tree' structures.
impl ActivationOperation {
    pub fn apply(&self, inputs: &[f32]) -> f32 {
        match self {
            ActivationOperation::Sigmoid => {
                let total = aggregate(inputs, |x| x);
                clamp(ONE / (ONE + (-total).exp()))
            }
            ActivationOperation::Tanh => {
                let total = aggregate(inputs, |x| x);
                clamp(total.tanh())
            }
            ActivationOperation::ReLU => clamp(inputs.iter().cloned().sum::<f32>().max(ZERO)),
            ActivationOperation::LeakyReLU => {
                let x = clamp(inputs.iter().cloned().sum::<f32>());
                if x > ZERO { x } else { clamp(HALF * x) }
            }
            ActivationOperation::ELU => {
                let x = clamp(inputs.iter().cloned().sum::<f32>());
                if x > ZERO {
                    x
                } else {
                    clamp(HALF * (x.exp() - ONE))
                }
            }
            ActivationOperation::Linear => clamp(inputs.iter().cloned().sum::<f32>()),
            ActivationOperation::Mish => {
                let x = clamp(inputs.iter().cloned().sum::<f32>());
                clamp(x * (x.exp().ln_1p().tanh()))
            }
            ActivationOperation::Swish => {
                let x = clamp(inputs.iter().cloned().sum::<f32>());
                clamp(x / (ONE + (-x).exp()))
            }
            ActivationOperation::Softplus => {
                let x = clamp(inputs.iter().cloned().sum::<f32>());
                clamp(x.exp().ln_1p())
            }
            ActivationOperation::Softmax => {
                let total = inputs.iter().cloned().map(|x| x.exp()).sum::<f32>();
                clamp(inputs.iter().cloned().map(|x| x.exp() / total).sum::<f32>())
            }
        }
    }
}

impl Op<f32> {
    pub fn weight() -> Self {
        let supplier = || random_provider::random::<f32>() * TWO - ONE;
        let operation = |inputs: &[f32], weight: &f32| clamp(inputs[0] * weight);
        let modifier = |current: &f32| {
            let diff = (random_provider::random::<f32>() * TWO - ONE) * TENTH;
            clamp(current + diff)
        };
        Op::MutableConst {
            name: "w",
            arity: 1.into(),
            value: supplier(),
            supplier: Arc::new(supplier),
            modifier: Arc::new(modifier),
            operation: Arc::new(operation),
        }
    }

    pub fn add() -> Self {
        Op::Fn(
            "add",
            2.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Add.apply(inputs)),
        )
    }

    pub fn sub() -> Self {
        Op::Fn(
            "sub",
            2.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Sub.apply(inputs)),
        )
    }

    pub fn mul() -> Self {
        Op::Fn(
            "mul",
            2.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Mul.apply(inputs)),
        )
    }

    pub fn div() -> Self {
        Op::Fn(
            "div",
            2.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Div.apply(inputs)),
        )
    }

    pub fn sum() -> Self {
        Op::Fn(
            "sum",
            Arity::Any,
            Arc::new(|inputs: &[f32]| MathOperation::Sum.apply(inputs)),
        )
    }

    pub fn diff() -> Self {
        Op::Fn(
            "diff",
            Arity::Any,
            Arc::new(|inputs: &[f32]| MathOperation::Diff.apply(inputs)),
        )
    }

    pub fn prod() -> Self {
        Op::Fn(
            "prod",
            Arity::Any,
            Arc::new(|inputs: &[f32]| MathOperation::Prod.apply(inputs)),
        )
    }

    pub fn neg() -> Self {
        Op::Fn(
            "neg",
            1.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Neg.apply(inputs)),
        )
    }

    pub fn pow() -> Self {
        Op::Fn(
            "pow",
            2.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Pow.apply(inputs)),
        )
    }

    pub fn sqrt() -> Self {
        Op::Fn(
            "sqrt",
            1.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Sqrt.apply(inputs)),
        )
    }

    pub fn abs() -> Self {
        Op::Fn(
            "abs",
            1.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Abs.apply(inputs)),
        )
    }

    pub fn exp() -> Self {
        Op::Fn(
            "exp",
            1.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Exp.apply(inputs)),
        )
    }

    pub fn log() -> Self {
        Op::Fn(
            "log",
            1.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Log.apply(inputs)),
        )
    }

    pub fn sin() -> Self {
        Op::Fn(
            "sin",
            1.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Sin.apply(inputs)),
        )
    }

    pub fn cos() -> Self {
        Op::Fn(
            "cos",
            1.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Cos.apply(inputs)),
        )
    }

    pub fn max() -> Self {
        Op::Fn(
            "max",
            Arity::Any,
            Arc::new(|inputs: &[f32]| MathOperation::Max.apply(inputs)),
        )
    }

    pub fn min() -> Self {
        Op::Fn(
            "min",
            Arity::Any,
            Arc::new(|inputs: &[f32]| MathOperation::Min.apply(inputs)),
        )
    }

    pub fn tan() -> Self {
        Op::Fn(
            "tan",
            1.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Tan.apply(inputs)),
        )
    }

    pub fn ceil() -> Self {
        Op::Fn(
            "ceil",
            1.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Ceil.apply(inputs)),
        )
    }

    pub fn floor() -> Self {
        Op::Fn(
            "floor",
            1.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Floor.apply(inputs)),
        )
    }

    pub fn sigmoid() -> Self {
        Op::Fn(
            "sigmoid",
            Arity::Any,
            Arc::new(|inputs: &[f32]| ActivationOperation::Sigmoid.apply(inputs)),
        )
    }

    pub fn tanh() -> Self {
        Op::Fn(
            "tanh",
            Arity::Any,
            Arc::new(|inputs: &[f32]| ActivationOperation::Tanh.apply(inputs)),
        )
    }

    pub fn relu() -> Self {
        Op::Fn(
            "relu",
            Arity::Any,
            Arc::new(|inputs: &[f32]| ActivationOperation::ReLU.apply(inputs)),
        )
    }

    pub fn leaky_relu() -> Self {
        Op::Fn(
            "l_relu",
            Arity::Any,
            Arc::new(|inputs: &[f32]| ActivationOperation::LeakyReLU.apply(inputs)),
        )
    }

    pub fn elu() -> Self {
        Op::Fn(
            "elu",
            Arity::Any,
            Arc::new(|inputs: &[f32]| ActivationOperation::ELU.apply(inputs)),
        )
    }

    pub fn linear() -> Self {
        Op::Fn(
            "linear",
            Arity::Any,
            Arc::new(|inputs: &[f32]| ActivationOperation::Linear.apply(inputs)),
        )
    }

    pub fn mish() -> Self {
        Op::Fn(
            "mish",
            Arity::Any,
            Arc::new(|inputs: &[f32]| ActivationOperation::Mish.apply(inputs)),
        )
    }

    pub fn swish() -> Self {
        Op::Fn(
            "swish",
            Arity::Any,
            Arc::new(|inputs: &[f32]| ActivationOperation::Swish.apply(inputs)),
        )
    }

    pub fn softplus() -> Self {
        Op::Fn(
            "softplus",
            Arity::Any,
            Arc::new(|inputs: &[f32]| ActivationOperation::Softplus.apply(inputs)),
        )
    }

    pub fn softmax() -> Self {
        Op::Fn(
            "softmax",
            Arity::Any,
            Arc::new(|inputs: &[f32]| ActivationOperation::Softmax.apply(inputs)),
        )
    }
}

/// Get a list of all the math operations.
pub fn math_ops() -> Vec<Op<f32>> {
    vec![
        Op::add(),
        Op::sub(),
        Op::mul(),
        Op::div(),
        Op::sum(),
        Op::prod(),
        Op::neg(),
        Op::diff(),
        Op::pow(),
        Op::sqrt(),
        Op::abs(),
        Op::exp(),
        Op::log(),
        Op::sin(),
        Op::cos(),
        Op::tan(),
        Op::ceil(),
        Op::floor(),
        Op::max(),
        Op::min(),
    ]
}

/// Get a list of all the activation operations.
pub fn activation_ops() -> Vec<Op<f32>> {
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
        Op::softmax(),
    ]
}

/// Get a list of all the operations.
pub fn all_ops() -> Vec<Op<f32>> {
    math_ops().into_iter().chain(activation_ops()).collect()
}
