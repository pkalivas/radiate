use super::{Arity, Operation};
use radiate::random_provider;
use std::sync::Arc;

const MAX_VALUE: f32 = 1e+5_f32;
const MIN_VALUE: f32 = -1e+5_f32;
const ONE: f32 = 1.0_f32;
const ZERO: f32 = 0.0_f32;
const TWO: f32 = 2.0_f32;
const HALF: f32 = 0.5_f32;

/// Clamp a value to the range [-1e+5, 1e+5]. Without this, values can quickly become
/// too large or too small to be useful.
fn clamp(value: f32) -> f32 {
    if value.is_nan() {
        return ZERO;
    }
    value.clamp(MIN_VALUE, MAX_VALUE)
}

/// Aggregate a slice of 'f32' values by summing them, then applying a function to the result.
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
                if x > ZERO {
                    x
                } else {
                    clamp(HALF * x)
                }
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
        }
    }
}

/// Implementations of the `Operation` trait for `f32`.
impl Operation<f32> {
    pub fn weight() -> Self {
        let supplier = || random_provider::random::<f32>() * TWO - ONE;
        let operation = |inputs: &[f32], weight: &f32| clamp(inputs[0] * weight);
        Operation::MutableConst {
            name: "w",
            arity: 1.into(),
            value: supplier(),
            get_value: Arc::new(supplier),
            operation: Arc::new(operation),
        }
    }

    pub fn add() -> Self {
        Operation::Fn(
            "add",
            2.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Add.apply(inputs)),
        )
    }

    pub fn sub() -> Self {
        Operation::Fn(
            "sub",
            2.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Sub.apply(inputs)),
        )
    }

    pub fn mul() -> Self {
        Operation::Fn(
            "mul",
            2.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Mul.apply(inputs)),
        )
    }

    pub fn div() -> Self {
        Operation::Fn(
            "div",
            2.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Div.apply(inputs)),
        )
    }

    pub fn sum() -> Self {
        Operation::Fn(
            "sum",
            Arity::Any,
            Arc::new(|inputs: &[f32]| MathOperation::Sum.apply(inputs)),
        )
    }

    pub fn prod() -> Self {
        Operation::Fn(
            "prod",
            Arity::Any,
            Arc::new(|inputs: &[f32]| MathOperation::Prod.apply(inputs)),
        )
    }

    pub fn neg() -> Self {
        Operation::Fn(
            "neg",
            1.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Neg.apply(inputs)),
        )
    }

    pub fn pow() -> Self {
        Operation::Fn(
            "pow",
            2.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Pow.apply(inputs)),
        )
    }

    pub fn sqrt() -> Self {
        Operation::Fn(
            "sqrt",
            1.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Sqrt.apply(inputs)),
        )
    }

    pub fn abs() -> Self {
        Operation::Fn(
            "abs",
            1.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Abs.apply(inputs)),
        )
    }

    pub fn exp() -> Self {
        Operation::Fn(
            "exp",
            1.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Exp.apply(inputs)),
        )
    }

    pub fn log() -> Self {
        Operation::Fn(
            "log",
            1.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Log.apply(inputs)),
        )
    }

    pub fn sin() -> Self {
        Operation::Fn(
            "sin",
            1.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Sin.apply(inputs)),
        )
    }

    pub fn cos() -> Self {
        Operation::Fn(
            "cos",
            1.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Cos.apply(inputs)),
        )
    }

    pub fn max() -> Self {
        Operation::Fn(
            "max",
            Arity::Any,
            Arc::new(|inputs: &[f32]| MathOperation::Max.apply(inputs)),
        )
    }

    pub fn min() -> Self {
        Operation::Fn(
            "min",
            Arity::Any,
            Arc::new(|inputs: &[f32]| MathOperation::Min.apply(inputs)),
        )
    }

    pub fn tan() -> Self {
        Operation::Fn(
            "tan",
            1.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Tan.apply(inputs)),
        )
    }

    pub fn ceil() -> Self {
        Operation::Fn(
            "ceil",
            1.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Ceil.apply(inputs)),
        )
    }

    pub fn floor() -> Self {
        Operation::Fn(
            "floor",
            1.into(),
            Arc::new(|inputs: &[f32]| MathOperation::Floor.apply(inputs)),
        )
    }

    pub fn sigmoid() -> Self {
        Operation::Fn(
            "sigmoid",
            Arity::Any,
            Arc::new(|inputs: &[f32]| ActivationOperation::Sigmoid.apply(inputs)),
        )
    }

    pub fn tanh() -> Self {
        Operation::Fn(
            "tanh",
            Arity::Any,
            Arc::new(|inputs: &[f32]| ActivationOperation::Tanh.apply(inputs)),
        )
    }

    pub fn relu() -> Self {
        Operation::Fn(
            "relu",
            Arity::Any,
            Arc::new(|inputs: &[f32]| ActivationOperation::ReLU.apply(inputs)),
        )
    }

    pub fn leaky_relu() -> Self {
        Operation::Fn(
            "leaky_relu",
            Arity::Any,
            Arc::new(|inputs: &[f32]| ActivationOperation::LeakyReLU.apply(inputs)),
        )
    }

    pub fn elu() -> Self {
        Operation::Fn(
            "elu",
            Arity::Any,
            Arc::new(|inputs: &[f32]| ActivationOperation::ELU.apply(inputs)),
        )
    }

    pub fn linear() -> Self {
        Operation::Fn(
            "linear",
            Arity::Any,
            Arc::new(|inputs: &[f32]| ActivationOperation::Linear.apply(inputs)),
        )
    }

    pub fn mish() -> Self {
        Operation::Fn(
            "mish",
            Arity::Any,
            Arc::new(|inputs: &[f32]| ActivationOperation::Mish.apply(inputs)),
        )
    }

    pub fn swish() -> Self {
        Operation::Fn(
            "swish",
            Arity::Any,
            Arc::new(|inputs: &[f32]| ActivationOperation::Swish.apply(inputs)),
        )
    }

    pub fn softplus() -> Self {
        Operation::Fn(
            "softplus",
            Arity::Any,
            Arc::new(|inputs: &[f32]| ActivationOperation::Softplus.apply(inputs)),
        )
    }
}

/// Get a list of all the math operations.
pub fn get_math_operations() -> Vec<Operation<f32>> {
    vec![
        Operation::add(),
        Operation::sub(),
        Operation::mul(),
        Operation::div(),
        Operation::sum(),
        Operation::prod(),
        Operation::neg(),
        Operation::pow(),
        Operation::sqrt(),
        Operation::abs(),
        Operation::exp(),
        Operation::log(),
        Operation::sin(),
        Operation::cos(),
        Operation::tan(),
        Operation::ceil(),
        Operation::floor(),
        Operation::max(),
        Operation::min(),
    ]
}

/// Get a list of all the activation operations.
pub fn get_activation_operations() -> Vec<Operation<f32>> {
    vec![
        Operation::sigmoid(),
        Operation::tanh(),
        Operation::relu(),
        Operation::leaky_relu(),
        Operation::elu(),
        Operation::linear(),
        Operation::mish(),
        Operation::swish(),
        Operation::softplus(),
    ]
}

/// Get a list of all the operations.
pub fn get_all_operations() -> Vec<Operation<f32>> {
    let mut operations = get_math_operations();
    operations.extend(get_activation_operations());
    operations
}
