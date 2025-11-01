use super::Op;
use crate::{Arity, ops::op_names};
use radiate_core::random_provider;

pub(super) const MAX_VALUE: f32 = 1e+10_f32;
pub(super) const MIN_VALUE: f32 = -1e+10_f32;
pub(super) const ONE: f32 = 1.0_f32;
pub(super) const ZERO: f32 = 0.0_f32;
pub(super) const TWO: f32 = 2.0_f32;
pub(super) const HALF: f32 = 0.5_f32;
pub(super) const TENTH: f32 = 0.1_f32;

/// Clamp a value to the range [-1e+10, 1e+10]. Without this, values can quickly become
/// too large or too small to be useful.
pub(super) const fn clamp(value: f32) -> f32 {
    if value.is_nan() {
        return ZERO;
    }

    value.clamp(MIN_VALUE, MAX_VALUE)
}

/// Aggregate a slice of 'f32' values by summing them, then applying a function to the result.
/// There usually arent too many inputs, so we can use an if statement to handle a few of the
/// common cases - vals with a len <= 5.
pub(super) fn aggregate(vals: &[f32]) -> f32 {
    let len = vals.len();
    if len == 0 {
        return ZERO;
    } else if len == 1 {
        return vals[0];
    } else if len == 2 {
        return vals[0] + vals[1];
    } else if len == 3 {
        return vals[0] + vals[1] + vals[2];
    } else if len == 4 {
        return vals[0] + vals[1] + vals[2] + vals[3];
    } else if len == 5 {
        return vals[0] + vals[1] + vals[2] + vals[3] + vals[4];
    }

    vals.iter().cloned().sum::<f32>()
}

#[inline]
const fn add(vals: &[f32]) -> f32 {
    clamp(vals[0] + vals[1])
}

#[inline]
const fn sub(vals: &[f32]) -> f32 {
    clamp(vals[0] - vals[1])
}

#[inline]
const fn mul(vals: &[f32]) -> f32 {
    clamp(vals[0] * vals[1])
}

#[inline]
const fn div(vals: &[f32]) -> f32 {
    if vals[1].abs() < MIN_VALUE {
        clamp(vals[0] / ONE)
    } else {
        clamp(vals[0] / vals[1])
    }
}

#[inline]
const fn neg(vals: &[f32]) -> f32 {
    clamp(-vals[0])
}

#[inline]
const fn abs(vals: &[f32]) -> f32 {
    clamp(vals[0].abs())
}

#[inline]
const fn ceil(vals: &[f32]) -> f32 {
    clamp(vals[0].ceil())
}

#[inline]
const fn floor(vals: &[f32]) -> f32 {
    clamp(vals[0].floor())
}

pub enum AggregateOperations {
    Sum,
    Prod,
    Diff,
    Pow,
    Sqrt,
    Exp,
    Log,
    Sin,
    Cos,
    Tan,
    Max,
    Min,
}

/// Implementations of the [MathOperation] enum. These are the basic math operations.
/// Each operation takes a slice of `f32` values and returns a single `f32` value.
impl AggregateOperations {
    pub fn apply(&self, inputs: &[f32]) -> f32 {
        match self {
            AggregateOperations::Sum => clamp(aggregate(inputs)),
            AggregateOperations::Diff => clamp(inputs.iter().cloned().fold(ZERO, |acc, x| acc - x)),
            AggregateOperations::Prod => clamp(inputs.iter().product()),
            AggregateOperations::Pow => clamp(inputs[0].powf(inputs[1])),
            AggregateOperations::Sqrt => clamp(inputs[0].sqrt()),
            AggregateOperations::Exp => clamp(inputs[0].exp()),
            AggregateOperations::Log => clamp(if inputs[0] > ZERO {
                inputs[0].ln()
            } else {
                ZERO
            }),
            AggregateOperations::Sin => clamp(inputs[0].sin()),
            AggregateOperations::Cos => clamp(inputs[0].cos()),
            AggregateOperations::Tan => clamp(inputs[0].tan()),
            AggregateOperations::Max => clamp(inputs.iter().cloned().fold(MIN_VALUE, f32::max)),
            AggregateOperations::Min => clamp(inputs.iter().cloned().fold(MAX_VALUE, f32::min)),
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

/// Implementations of the [ActivationOperation] enum. These are the basic activation functions used
/// in neural networks. However, they are particularly useful in this context because they can
/// accept any number of inputs. Thus, they act as reducers or aggregates and are a key part of
/// being able to define complex 'Graph' and 'Tree' structures.
impl ActivationOperation {
    #[inline]
    pub fn apply(&self, inputs: &[f32]) -> f32 {
        match self {
            ActivationOperation::Sigmoid => {
                let total = aggregate(inputs);
                clamp(ONE / (ONE + (-total).exp()))
            }
            ActivationOperation::Tanh => {
                let total = aggregate(inputs);
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
        }
    }
}

impl Op<f32> {
    pub fn weight() -> Self {
        Self::weight_with(random_provider::random::<f32>() * TWO - ONE)
    }

    pub fn weight_with(value: f32) -> Self {
        let supplier = || random_provider::random::<f32>() * TWO - ONE;
        let operation = |inputs: &[f32], weight: &f32| clamp(inputs[0] * weight);
        let modifier = |current: &f32| {
            let diff = (random_provider::random::<f32>() * TWO - ONE) * TENTH;
            clamp(current + diff)
        };

        Op::MutableConst {
            name: op_names::WEIGHT,
            arity: 1.into(),
            value: clamp(value),
            supplier,
            modifier,
            operation,
        }
    }

    pub fn add() -> Self {
        Op::Fn(op_names::ADD, 2.into(), add)
    }

    pub fn sub() -> Self {
        Op::Fn(op_names::SUB, 2.into(), sub)
    }

    pub fn mul() -> Self {
        Op::Fn(op_names::MUL, 2.into(), mul)
    }

    pub fn div() -> Self {
        Op::Fn(op_names::DIV, 2.into(), div)
    }

    pub fn sum() -> Self {
        Op::Fn(op_names::SUM, Arity::Any, |inputs: &[f32]| {
            AggregateOperations::Sum.apply(inputs)
        })
    }

    pub fn diff() -> Self {
        Op::Fn(op_names::DIFF, Arity::Any, |inputs: &[f32]| {
            AggregateOperations::Diff.apply(inputs)
        })
    }

    pub fn prod() -> Self {
        Op::Fn(op_names::PROD, Arity::Any, |inputs: &[f32]| {
            AggregateOperations::Prod.apply(inputs)
        })
    }

    pub fn neg() -> Self {
        Op::Fn(op_names::NEG, 1.into(), neg)
    }

    pub fn pow() -> Self {
        Op::Fn(op_names::POW, 2.into(), |inputs: &[f32]| {
            AggregateOperations::Pow.apply(inputs)
        })
    }

    pub fn sqrt() -> Self {
        Op::Fn(op_names::SQRT, 1.into(), |inputs: &[f32]| {
            AggregateOperations::Sqrt.apply(inputs)
        })
    }

    pub fn abs() -> Self {
        Op::Fn(op_names::ABS, 1.into(), abs)
    }

    pub fn exp() -> Self {
        Op::Fn(op_names::EXP, 1.into(), |inputs: &[f32]| {
            AggregateOperations::Exp.apply(inputs)
        })
    }

    pub fn log() -> Self {
        Op::Fn(op_names::LOG, 1.into(), |inputs: &[f32]| {
            AggregateOperations::Log.apply(inputs)
        })
    }

    pub fn sin() -> Self {
        Op::Fn(op_names::SIN, 1.into(), |inputs: &[f32]| {
            AggregateOperations::Sin.apply(inputs)
        })
    }

    pub fn cos() -> Self {
        Op::Fn(op_names::COS, 1.into(), |inputs: &[f32]| {
            AggregateOperations::Cos.apply(inputs)
        })
    }

    pub fn max() -> Self {
        Op::Fn(op_names::MAX, Arity::Any, |inputs: &[f32]| {
            AggregateOperations::Max.apply(inputs)
        })
    }

    pub fn min() -> Self {
        Op::Fn(op_names::MIN, Arity::Any, |inputs: &[f32]| {
            AggregateOperations::Min.apply(inputs)
        })
    }

    pub fn tan() -> Self {
        Op::Fn(op_names::TAN, 1.into(), |inputs: &[f32]| {
            AggregateOperations::Tan.apply(inputs)
        })
    }

    pub fn ceil() -> Self {
        Op::Fn(op_names::CEIL, 1.into(), ceil)
    }

    pub fn floor() -> Self {
        Op::Fn(op_names::FLOOR, 1.into(), floor)
    }

    pub fn sigmoid() -> Self {
        Op::Fn(op_names::SIGMOID, Arity::Any, |inputs: &[f32]| {
            ActivationOperation::Sigmoid.apply(inputs)
        })
    }

    pub fn tanh() -> Self {
        Op::Fn(op_names::TANH, Arity::Any, |inputs: &[f32]| {
            ActivationOperation::Tanh.apply(inputs)
        })
    }

    pub fn relu() -> Self {
        Op::Fn(op_names::RELU, Arity::Any, |inputs: &[f32]| {
            ActivationOperation::ReLU.apply(inputs)
        })
    }

    pub fn leaky_relu() -> Self {
        Op::Fn(op_names::LEAKY_RELU, Arity::Any, |inputs: &[f32]| {
            ActivationOperation::LeakyReLU.apply(inputs)
        })
    }

    pub fn elu() -> Self {
        Op::Fn(op_names::ELU, Arity::Any, |inputs: &[f32]| {
            ActivationOperation::ELU.apply(inputs)
        })
    }

    pub fn linear() -> Self {
        Op::Fn(op_names::LINEAR, Arity::Any, |inputs: &[f32]| {
            ActivationOperation::Linear.apply(inputs)
        })
    }

    pub fn mish() -> Self {
        Op::Fn(op_names::MISH, Arity::Any, |inputs: &[f32]| {
            ActivationOperation::Mish.apply(inputs)
        })
    }

    pub fn swish() -> Self {
        Op::Fn(op_names::SWISH, Arity::Any, |inputs: &[f32]| {
            ActivationOperation::Swish.apply(inputs)
        })
    }

    pub fn softplus() -> Self {
        Op::Fn(op_names::SOFTPLUS, Arity::Any, |inputs: &[f32]| {
            ActivationOperation::Softplus.apply(inputs)
        })
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
    ]
}

/// Get a list of all the operations.
pub fn all_ops() -> Vec<Op<f32>> {
    math_ops().into_iter().chain(activation_ops()).collect()
}

#[cfg(test)]
mod tests {
    use crate::Eval;

    use super::*;
    use std::f32;

    #[inline]
    fn approx(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() <= eps
    }

    #[test]
    fn clamp_behaves_as_specified() {
        assert_eq!(super::clamp(f32::NAN), ZERO);
        assert_eq!(super::clamp(1e20_f32), MAX_VALUE);
        assert_eq!(super::clamp(-1e20_f32), MIN_VALUE);
        assert_eq!(super::clamp(123.456), 123.456);
    }

    #[test]
    fn math_div_near_zero_clamps_large_quotient() {
        let xs = [10.0, 1e-12_f32];
        let y = Op::div().eval(&xs);
        assert_eq!(
            y, MAX_VALUE,
            "huge quotient should clamp to MAX_VALUE with current code"
        );
    }

    #[test]
    fn math_sum_prod_diff_pow_sqrt_abs() {
        let xs = [2.0, 3.0, 4.0];
        assert_eq!(AggregateOperations::Sum.apply(&xs), 9.0);
        assert_eq!(AggregateOperations::Prod.apply(&xs), 24.0);
        // Diff is left fold from ZERO: (((0-2)-3)-4) = -9
        assert_eq!(AggregateOperations::Diff.apply(&xs), -9.0);

        let p = AggregateOperations::Pow.apply(&[3.0, 2.0]);
        assert_eq!(p, 9.0);

        assert_eq!(AggregateOperations::Sqrt.apply(&[9.0]), 3.0);
    }

    #[test]
    fn math_exp_log_trig_rounding() {
        let e = AggregateOperations::Exp.apply(&[1.0]);
        assert!(approx(e, f32::consts::E, 1e-5), "exp(1) ~= e");

        // log on <=0 becomes NaN, then clamp -> 0.0
        assert_eq!(AggregateOperations::Log.apply(&[0.0]), 0.0);
        assert_eq!(AggregateOperations::Log.apply(&[-1.0]), 0.0);

        let s = AggregateOperations::Sin.apply(&[f32::consts::PI / 2.0]);
        assert!(approx(s, 1.0, 1e-5));

        let c = AggregateOperations::Cos.apply(&[0.0]);
        assert!(approx(c, 1.0, 1e-5));

        let t = AggregateOperations::Tan.apply(&[0.0]);
        assert!(approx(t, 0.0, 1e-6));
    }

    #[test]
    fn math_max_min_variadic_including_empty_behavior() {
        let xs = [1.5, -2.0, 7.25, 3.0];
        let mx = AggregateOperations::Max.apply(&xs);
        let mn = AggregateOperations::Min.apply(&xs);
        assert_eq!(mx, 7.25);
        assert_eq!(mn, -2.0);

        let empty: [f32; 0] = [];
        assert_eq!(AggregateOperations::Max.apply(&empty), MIN_VALUE);
        assert_eq!(AggregateOperations::Min.apply(&empty), MAX_VALUE);
    }

    #[test]
    fn act_sigmoid_on_sum() {
        // sum = 1.0 -> sigmoid(1) ~ 0.731
        let xs = [2.0, -1.0];
        let y = ActivationOperation::Sigmoid.apply(&xs);
        assert!(y > 0.73 && y < 0.74, "got {y}");
    }

    #[test]
    fn act_tanh_on_sum() {
        let xs = [2.0, -0.5]; // sum = 1.5 -> tanh(1.5) ~ 0.9051
        let y = ActivationOperation::Tanh.apply(&xs);
        assert!(y > 0.90 && y < 0.91, "got {y}");
    }

    #[test]
    fn act_relu_and_leaky_and_elu_match_current_params() {
        // ReLU(sum)
        let xs = [-1.0, 0.25, 0.25]; // sum = -0.5
        assert_eq!(ActivationOperation::ReLU.apply(&xs), 0.0);

        // LeakyReLU uses slope HALF (=0.5) per current code
        let xs2 = [-0.6];
        let y2 = ActivationOperation::LeakyReLU.apply(&xs2);
        assert_eq!(y2, -0.3);

        // ELU uses alpha HALF (=0.5) currently
        let xs3 = [-1.0];
        let y3 = ActivationOperation::ELU.apply(&xs3);
        // 0.5 * (exp(-1) - 1) ~= -0.316060...
        assert!(approx(y3, 0.5 * (f32::consts::E.powf(-1.0) - 1.0), 1e-6));
    }

    #[test]
    fn act_linear_mish_swish_softplus() {
        // Linear is sum
        let xs = [1.0, 2.0, 3.0];
        assert_eq!(ActivationOperation::Linear.apply(&xs), 6.0);

        // Mish ~ x * tanh(ln(1+exp(x))) at sum(x)
        let x = 1.5_f32;
        let mish_ref = x * ((x.exp().ln_1p()).tanh());
        let mish_y = ActivationOperation::Mish.apply(&[x]);
        assert!(approx(mish_y, mish_ref, 1e-6));

        // Swish ~ x * sigmoid(x); implementation uses x / (1 + exp(-x))
        let sw = ActivationOperation::Swish.apply(&[x]);
        let sw_ref = x / (1.0 + (-x).exp());
        assert!(approx(sw, sw_ref, 1e-6));

        // Softplus = ln(1 + exp(x))
        let sp = ActivationOperation::Softplus.apply(&[x]);
        let sp_ref = x.exp().ln_1p();
        assert!(approx(sp, sp_ref, 1e-6));
    }

    #[test]
    fn weight_op_runs_and_is_clamped() {
        let w = Op::<f32>::weight();
        if let Op::MutableConst {
            operation, value, ..
        } = &w
        {
            let out = (operation)(&[0.5], value);
            assert!(out.is_finite());
            assert!(out <= MAX_VALUE && out >= MIN_VALUE);
        } else {
            panic!("weight() did not return MutableConst as expected");
        }
    }
}
