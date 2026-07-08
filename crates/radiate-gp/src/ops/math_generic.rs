use super::Op;
use crate::{
    Arity,
    ops::{Param, op_names},
};
use radiate_core::random_provider;

use super::GpFloat;

/// Clamp a value to the range [-MAX_VALUE, MAX_VALUE]. Without this, values can quickly
/// become too large or too small to be useful.
#[inline]
fn clamp<F: GpFloat>(value: F) -> F {
    if value.is_nan() {
        return F::ZERO;
    }
    value.clamp(-F::MAX_VALUE, F::MAX_VALUE)
}

/// Aggregate a slice of values by summing them. There usually aren't too many inputs, so we
/// can use an if statement to handle a few of the common cases - vals with a len <= 5.
#[inline]
fn aggregate<F: GpFloat>(vals: &[F]) -> F {
    match vals {
        [] => F::ZERO,
        [a] => *a,
        [a, b] => *a + *b,
        [a, b, c] => *a + *b + *c,
        [a, b, c, d] => *a + *b + *c + *d,
        [a, b, c, d, e] => *a + *b + *c + *d + *e,
        _ => vals.iter().copied().fold(F::ZERO, |acc, x| acc + x),
    }
}

#[inline]
fn add<F: GpFloat>(vals: &[F]) -> F {
    clamp(vals[0] + vals[1])
}

#[inline]
fn sub<F: GpFloat>(vals: &[F]) -> F {
    clamp(vals[0] - vals[1])
}

#[inline]
fn mul<F: GpFloat>(vals: &[F]) -> F {
    clamp(vals[0] * vals[1])
}

#[inline]
fn div<F: GpFloat>(vals: &[F]) -> F {
    if vals[1].abs() < F::EPS {
        F::ONE
    } else {
        clamp(vals[0] / vals[1])
    }
}

#[inline]
fn neg<F: GpFloat>(vals: &[F]) -> F {
    clamp(-vals[0])
}

#[inline]
fn abs<F: GpFloat>(vals: &[F]) -> F {
    clamp(vals[0].abs())
}

#[inline]
fn ceil<F: GpFloat>(vals: &[F]) -> F {
    clamp(vals[0].ceil())
}

#[inline]
fn floor<F: GpFloat>(vals: &[F]) -> F {
    clamp(vals[0].floor())
}

#[inline]
fn logsumexp<F: GpFloat>(xs: &[F]) -> F {
    let mut m = F::neg_infinity();
    let mut s = F::ZERO;

    for &x in xs {
        if x > m {
            m = x;
        }
    }

    for &x in xs {
        s = s + (x - m).exp();
    }

    m + s.ln()
}

#[inline]
fn softplus_stable<F: GpFloat>(x: F) -> F {
    // x already clamped
    let threshold = F::from(20.0).unwrap();
    if x > threshold {
        x
    } else if x < -threshold {
        x.exp() // ~0
    } else {
        (F::ONE + x.exp()).ln()
    }
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
    LogSumExp,
}

/// Implementations of the enum. These are the basic math operations.
/// Each operation takes a slice of `F` values and returns a single `F` value.
impl AggregateOperations {
    pub fn apply<F: GpFloat>(&self, inputs: &[F]) -> F {
        match self {
            AggregateOperations::Sum => clamp(aggregate(inputs)),
            AggregateOperations::Diff => {
                clamp(inputs.iter().copied().fold(F::ZERO, |acc, x| acc - x))
            }
            AggregateOperations::Prod => {
                clamp(inputs.iter().copied().fold(F::ONE, |acc, x| acc * x))
            }
            AggregateOperations::Pow => clamp(inputs[0].powf(inputs[1])),
            AggregateOperations::Sqrt => clamp(inputs[0].sqrt()),
            AggregateOperations::Exp => clamp(inputs[0].exp()),
            AggregateOperations::Log => clamp(if inputs[0] > F::ZERO {
                inputs[0].ln()
            } else {
                F::ZERO
            }),
            AggregateOperations::Sin => clamp(inputs[0].sin()),
            AggregateOperations::Cos => clamp(inputs[0].cos()),
            AggregateOperations::Tan => clamp(inputs[0].tan()),
            AggregateOperations::Max => clamp(inputs.iter().copied().fold(-F::MAX_VALUE, F::max)),
            AggregateOperations::Min => clamp(inputs.iter().copied().fold(F::MAX_VALUE, F::min)),
            AggregateOperations::LogSumExp => clamp(logsumexp(inputs)),
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
    pub fn apply<F: GpFloat>(&self, inputs: &[F]) -> F {
        match self {
            ActivationOperation::Sigmoid => {
                let total = aggregate(inputs);
                clamp(F::ONE / (F::ONE + (-total).exp()))
            }
            ActivationOperation::Tanh => {
                let total = aggregate(inputs);
                clamp(total.tanh())
            }
            ActivationOperation::ReLU => clamp(aggregate(inputs).max(F::ZERO)),
            ActivationOperation::LeakyReLU => {
                let x = clamp(aggregate(inputs));
                if x > F::ZERO { x } else { clamp(F::HALF * x) }
            }
            ActivationOperation::ELU => {
                let x = clamp(aggregate(inputs));
                if x > F::ZERO {
                    x
                } else {
                    clamp(F::HALF * (x.exp() - F::ONE))
                }
            }
            ActivationOperation::Linear => clamp(aggregate(inputs)),
            ActivationOperation::Mish => {
                let x = clamp(aggregate(inputs));
                clamp(x * (x.exp().ln_1p().tanh()))
            }
            ActivationOperation::Swish => {
                let x = clamp(aggregate(inputs));
                clamp(x / (F::ONE + (-x).exp()))
            }
            ActivationOperation::Softplus => softplus_stable(clamp(aggregate(inputs))),
        }
    }
}

impl<F: GpFloat> Op<F> {
    pub fn weight() -> Self {
        Self::weight_with(random_provider::range(-F::ONE..F::ONE))
    }

    pub fn weight_with(value: F) -> Self {
        let supplier = || random_provider::range(-F::ONE..F::ONE);

        let operation = |inputs: &[F], weight: &F| clamp(inputs[0] * *weight);

        let modifier = |current: &mut F| {
            let diff = random_provider::range(-F::ONE..F::ONE) * F::TENTH;
            *current = clamp(*current + diff);
        };

        Op::Value(
            op_names::WEIGHT,
            1.into(),
            Param::new(clamp(value), supplier, modifier),
            operation,
        )
    }

    pub fn add() -> Self {
        Op::Fn(op_names::ADD, 2.into(), add::<F>)
    }

    pub fn sub() -> Self {
        Op::Fn(op_names::SUB, 2.into(), sub::<F>)
    }

    pub fn mul() -> Self {
        Op::Fn(op_names::MUL, 2.into(), mul::<F>)
    }

    pub fn div() -> Self {
        Op::Fn(op_names::DIV, 2.into(), div::<F>)
    }

    pub fn sum() -> Self {
        Op::Fn(op_names::SUM, Arity::Any, |inputs: &[F]| {
            AggregateOperations::Sum.apply(inputs)
        })
    }

    pub fn diff() -> Self {
        Op::Fn(op_names::DIFF, Arity::Any, |inputs: &[F]| {
            AggregateOperations::Diff.apply(inputs)
        })
    }

    pub fn prod() -> Self {
        Op::Fn(op_names::PROD, Arity::Any, |inputs: &[F]| {
            AggregateOperations::Prod.apply(inputs)
        })
    }

    pub fn neg() -> Self {
        Op::Fn(op_names::NEG, 1.into(), neg::<F>)
    }

    pub fn pow() -> Self {
        Op::Fn(op_names::POW, 2.into(), |inputs: &[F]| {
            AggregateOperations::Pow.apply(inputs)
        })
    }

    pub fn sqrt() -> Self {
        Op::Fn(op_names::SQRT, 1.into(), |inputs: &[F]| {
            AggregateOperations::Sqrt.apply(inputs)
        })
    }

    pub fn abs() -> Self {
        Op::Fn(op_names::ABS, 1.into(), abs::<F>)
    }

    pub fn exp() -> Self {
        Op::Fn(op_names::EXP, 1.into(), |inputs: &[F]| {
            AggregateOperations::Exp.apply(inputs)
        })
    }

    pub fn log() -> Self {
        Op::Fn(op_names::LOG, 1.into(), |inputs: &[F]| {
            AggregateOperations::Log.apply(inputs)
        })
    }

    pub fn sin() -> Self {
        Op::Fn(op_names::SIN, 1.into(), |inputs: &[F]| {
            AggregateOperations::Sin.apply(inputs)
        })
    }

    pub fn cos() -> Self {
        Op::Fn(op_names::COS, 1.into(), |inputs: &[F]| {
            AggregateOperations::Cos.apply(inputs)
        })
    }

    pub fn max() -> Self {
        Op::Fn(op_names::MAX, Arity::Any, |inputs: &[F]| {
            AggregateOperations::Max.apply(inputs)
        })
    }

    pub fn min() -> Self {
        Op::Fn(op_names::MIN, Arity::Any, |inputs: &[F]| {
            AggregateOperations::Min.apply(inputs)
        })
    }

    pub fn tan() -> Self {
        Op::Fn(op_names::TAN, 1.into(), |inputs: &[F]| {
            AggregateOperations::Tan.apply(inputs)
        })
    }

    pub fn ceil() -> Self {
        Op::Fn(op_names::CEIL, 1.into(), ceil::<F>)
    }

    pub fn floor() -> Self {
        Op::Fn(op_names::FLOOR, 1.into(), floor::<F>)
    }

    pub fn sigmoid() -> Self {
        Op::Fn(op_names::SIGMOID, Arity::Any, |inputs: &[F]| {
            ActivationOperation::Sigmoid.apply(inputs)
        })
    }

    pub fn tanh() -> Self {
        Op::Fn(op_names::TANH, Arity::Any, |inputs: &[F]| {
            ActivationOperation::Tanh.apply(inputs)
        })
    }

    pub fn relu() -> Self {
        Op::Fn(op_names::RELU, Arity::Any, |inputs: &[F]| {
            ActivationOperation::ReLU.apply(inputs)
        })
    }

    pub fn leaky_relu() -> Self {
        Op::Fn(op_names::LEAKY_RELU, Arity::Any, |inputs: &[F]| {
            ActivationOperation::LeakyReLU.apply(inputs)
        })
    }

    pub fn elu() -> Self {
        Op::Fn(op_names::ELU, Arity::Any, |inputs: &[F]| {
            ActivationOperation::ELU.apply(inputs)
        })
    }

    pub fn linear() -> Self {
        Op::Fn(op_names::LINEAR, Arity::Any, |inputs: &[F]| {
            ActivationOperation::Linear.apply(inputs)
        })
    }

    pub fn mish() -> Self {
        Op::Fn(op_names::MISH, Arity::Any, |inputs: &[F]| {
            ActivationOperation::Mish.apply(inputs)
        })
    }

    pub fn swish() -> Self {
        Op::Fn(op_names::SWISH, Arity::Any, |inputs: &[F]| {
            ActivationOperation::Swish.apply(inputs)
        })
    }

    pub fn softplus() -> Self {
        Op::Fn(op_names::SOFTPLUS, Arity::Any, |inputs: &[F]| {
            ActivationOperation::Softplus.apply(inputs)
        })
    }

    pub fn logsumexp() -> Self {
        Op::Fn(op_names::LOGSUMEXP, Arity::Exact(2), |inputs: &[F]| {
            AggregateOperations::LogSumExp.apply(inputs)
        })
    }
}

/// Get a list of all the math operations.
pub fn math_ops<F: GpFloat>() -> Vec<Op<F>> {
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
        Op::logsumexp(),
    ]
}

/// Get a list of all the activation operations.
pub fn activation_ops<F: GpFloat>() -> Vec<Op<F>> {
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

pub fn edge_ops<F: GpFloat>() -> Vec<Op<F>> {
    vec![Op::weight(), Op::identity()]
}

/// Get a list of all the operations.
pub fn all_ops<F: GpFloat>() -> Vec<Op<F>> {
    math_ops()
        .into_iter()
        .chain(activation_ops())
        .chain(edge_ops())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Eval;

    #[inline]
    fn approx<F: GpFloat>(a: F, b: F, eps: F) -> bool {
        (a - b).abs() <= eps
    }

    #[test]
    fn clamp_behaves_as_specified_f32() {
        assert_eq!(super::clamp(f32::NAN), 0.0_f32);
        assert_eq!(super::clamp(1e20_f32), f32::MAX_VALUE);
        assert_eq!(super::clamp(-1e20_f32), -f32::MAX_VALUE);
        assert_eq!(super::clamp(123.456_f32), 123.456_f32);
    }

    #[test]
    fn clamp_behaves_as_specified_f64() {
        assert_eq!(super::clamp(f64::NAN), 0.0_f64);
        assert_eq!(super::clamp(1e20_f64), f64::MAX_VALUE);
        assert_eq!(super::clamp(-1e20_f64), -f64::MAX_VALUE);
        assert_eq!(super::clamp(123.456_f64), 123.456_f64);
    }

    #[test]
    fn math_div_by_zero_behavior_f32() {
        let xs = [10.0_f32, 1e-12_f32];
        let y = Op::div().eval(&xs);
        assert_eq!(y, 1.0);
    }

    #[test]
    fn math_div_by_zero_behavior_f64() {
        let xs = [10.0_f64, 1e-20_f64];
        let y = Op::div().eval(&xs);
        assert_eq!(y, 1.0);
    }

    #[test]
    fn math_sum_prod_diff_pow_sqrt_abs() {
        let xs = [2.0_f32, 3.0, 4.0];
        assert_eq!(AggregateOperations::Sum.apply(&xs), 9.0);
        assert_eq!(AggregateOperations::Prod.apply(&xs), 24.0);
        assert_eq!(AggregateOperations::Diff.apply(&xs), -9.0);

        let p = AggregateOperations::Pow.apply(&[3.0_f32, 2.0]);
        assert_eq!(p, 9.0);

        assert_eq!(AggregateOperations::Sqrt.apply(&[9.0_f32]), 3.0);
    }

    #[test]
    fn act_relu_and_leaky_and_elu_match_current_params() {
        let xs = [-1.0_f32, 0.25, 0.25];
        assert_eq!(ActivationOperation::ReLU.apply(&xs), 0.0);

        let xs2 = [-0.6_f32];
        let y2 = ActivationOperation::LeakyReLU.apply(&xs2);
        assert_eq!(y2, -0.3);

        let xs3 = [-1.0_f32];
        let y3 = ActivationOperation::ELU.apply(&xs3);
        assert!(approx(
            y3,
            0.5 * (std::f32::consts::E.powf(-1.0) - 1.0),
            1e-6
        ));
    }

    #[test]
    fn softplus_is_stable_for_large_x_f64() {
        let big = 1e9_f64;
        let y = ActivationOperation::Softplus.apply(&[big]);
        assert!(y.is_finite());
        assert!(y > 1e8);
    }
}
