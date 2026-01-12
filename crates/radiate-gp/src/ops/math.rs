use std::{ops::Add, sync::Arc, vec};

use super::Op;
use crate::{
    Arity,
    ops::{OpData, OpValue, op_names},
};
use radiate_core::{chromosomes::NumericAllele, random_provider};

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
#[inline]
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
    } else if len == 6 {
        return vals[0] + vals[1] + vals[2] + vals[3] + vals[4] + vals[5];
    } else if len == 7 {
        return vals[0] + vals[1] + vals[2] + vals[3] + vals[4] + vals[5] + vals[6];
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
        let supplier = |_: &OpData<f32>| OpData::Unit(random_provider::random::<f32>() * TWO - ONE);

        let operation = |inputs: &[f32], weight: &OpValue<f32>| {
            clamp(inputs[0] * weight.data().as_scalar().map_or(ZERO, |v| *v))
        };

        let modifier = |current: &mut OpData<f32>| {
            let diff = (random_provider::random::<f32>() * TWO - ONE) * TENTH;
            if let OpData::Unit(v) = current {
                *v = clamp(*v + diff);
            }
        };

        Op::Value(
            op_names::WEIGHT,
            1.into(),
            OpValue::new(OpData::Unit(clamp(value)), supplier, modifier),
            operation,
        )
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

    pub fn probability_table(dims: impl Into<Vec<usize>>) -> Self {
        let dims = dims.into();

        assert!(!dims.is_empty(), "table dims cannot be empty");
        assert!(dims.iter().all(|&d| d > 0), "dims must all be > 0");

        let supplier = |value: &OpData<f32>| match value {
            OpData::Array { dims, .. } => OpData::from((Arc::clone(dims), |_| {
                random_provider::random::<f32>() * TWO - ONE
            })),
            _ => OpData::Unit(random_provider::random::<f32>() * TWO - ONE),
        };

        let eval = |inputs: &[f32], val: &OpValue<f32>| {
            if let Some(dims) = val.dims() {
                println!("Inputs: {:?}, Dims: {:?}", inputs, dims);
                assert!(
                    inputs.len() == dims.len(),
                    "number of inputs must match table dimensions ({} != {})",
                    inputs.len(),
                    dims.len()
                );

                let mut index: usize = 0;
                match val.data() {
                    OpData::Array {
                        strides, values, ..
                    } => {
                        for i in 0..inputs.len() {
                            let stride = strides[i];

                            // round -> clamp into [0, d-1]
                            let dim = dims[i].max(1);

                            let mut v = (inputs[i].round() as isize).max(0) as usize;

                            if v >= dim {
                                v = dim - 1;
                            }

                            index = index.saturating_add(v.saturating_mul(stride));
                        }

                        values.get(index).copied().map(clamp).unwrap_or(ZERO)
                    }
                    _ => ZERO,
                }
            } else {
                ZERO
            }
        };

        let modifier = |current: &mut OpData<f32>| {
            if let OpData::Array { values, .. } = current {
                let values = Arc::make_mut(values);

                random_provider::with_rng(|rng| {
                    let indecies = rng.sample_indices(0..values.len(), values.len() / 10 + 1);
                    for &idx in &indecies {
                        let diff = (rng.random::<f32>() * TWO - ONE) * TENTH;
                        values[idx] = clamp(values[idx] + diff);
                    }
                });
            }
        };

        let data = OpData::from((dims.clone(), |_| {
            random_provider::random::<f32>() * TWO - ONE
        }));

        Op::Value(
            op_names::PROBABILITY_TABLE,
            Arity::Exact(dims.len()),
            OpValue::new(data, supplier, modifier),
            eval,
        )
    }
}

impl NumericAllele for Op<f32> {
    fn cast_as_f32(&self) -> Option<f32> {
        match self {
            Op::Const(_, value) => Some(*value),
            Op::Value(_, _, value, _) => value.data().as_scalar().copied(),
            _ => None,
        }
    }

    fn cast_as_i32(&self) -> Option<i32> {
        match self {
            Op::Const(_, value) => Some(*value as i32),
            Op::Value(_, _, value, _) => value.data().as_scalar().map(|v| *v as i32),
            _ => None,
        }
    }
}

impl Add for Op<f32> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Op::Value(name, arity, value, op), Op::Value(_, _, other_value, _)) => {
                match (value.data(), other_value.data()) {
                    (OpData::Unit(a), OpData::Unit(b)) => Op::Value(
                        radiate_utils::intern!(String::from(*name)),
                        *arity,
                        OpValue::new(
                            OpData::Unit(clamp(a + b)),
                            value.supplier(),
                            value.modifier(),
                        ),
                        *op,
                    ),
                    (OpData::Array { .. }, OpData::Array { .. }) => Op::Value(
                        radiate_utils::intern!(String::from(*name)),
                        *arity,
                        OpValue::new(
                            OpData::from((other_value.dims().unwrap().to_vec(), |idx| {
                                let a = value.data().as_array().unwrap()[idx];
                                let b = other_value.data().as_array().unwrap()[idx];
                                clamp(a + b)
                            })),
                            value.supplier(),
                            value.modifier(),
                        ),
                        *op,
                    ),
                    _ => rhs.clone(),
                }
            }
            _ => rhs.clone(),
        }
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

impl Op<f32> {
    pub fn log_likelihood_table(dims: impl Into<Vec<usize>>) -> Self {
        let dims = dims.into();
        assert!(dims.len() >= 1, "dims must include child axis");
        assert!(dims.iter().all(|&d| d > 0), "dims must all be > 0");

        // reuse your OpData::from((dims, f)) builder
        let data = OpData::from((dims.as_slice(), |_| {
            random_provider::random::<f32>() * TWO - ONE
        }));

        // resample: regenerate the whole table values, keep dims/strides
        let supplier = |value: &OpData<f32>| {
            let size = value.dims().unwrap().iter().product::<usize>();
            OpData::Array {
                values: Arc::from(
                    random_provider::vector::<f32>(size)
                        .into_iter()
                        .map(|v| v * TWO - ONE)
                        .collect::<Vec<f32>>(),
                ),
                strides: Arc::from(value.strides().unwrap().to_vec()),
                dims: Arc::from(value.dims().unwrap().to_vec()),
            }
        };

        // mutate: small noise to each entry (or you can mutate a subset)
        let modifier = |current: &mut OpData<f32>| {
            if let OpData::Array { values, .. } = current {
                let values = Arc::make_mut(values);
                random_provider::with_rng(|rng| {
                    for x in values.iter_mut() {
                        let diff = (rng.random::<f32>() * TWO - ONE) * TENTH;
                        *x = clamp(*x + diff);
                    }
                });
            }
        };

        // loglik: score(child) - logsumexp(scores over child-axis)
        let operation = |inputs: &[f32], val: &OpValue<f32>| -> f32 {
            let OpData::Array {
                values,
                strides,
                dims,
            } = val.data()
            else {
                return ZERO;
            };

            if inputs.len() != dims.len() {
                return ZERO;
            }

            // last axis is child
            let child_axis = dims.len() - 1;
            let child_states = dims[child_axis];
            let child_stride = strides[child_axis];

            // convert inputs -> indices, clamped into [0, dims[i]-1]
            let mut idxs = vec![0usize; dims.len()];
            for i in 0..dims.len() {
                let mut v = inputs[i].round() as isize;
                if v < 0 {
                    v = 0;
                }
                let v = v as usize;
                idxs[i] = v.min(dims[i].saturating_sub(1));
            }

            // base offset excluding child (set child=0)
            let mut base = 0usize;
            for i in 0..child_axis {
                base = base.saturating_add(idxs[i].saturating_mul(strides[i]));
            }

            let child = idxs[child_axis];

            // gather scores for this parent config across all child states
            // and compute logsumexp stably
            let mut max_score = f32::NEG_INFINITY;
            for k in 0..child_states {
                let pos = base.saturating_add(k.saturating_mul(child_stride));
                if let Some(&s) = values.get(pos) {
                    if s > max_score {
                        max_score = s;
                    }
                }
            }
            if !max_score.is_finite() {
                return ZERO;
            }

            let mut sum_exp = 0.0f32;
            for k in 0..child_states {
                let pos = base.saturating_add(k.saturating_mul(child_stride));
                let s = values.get(pos).copied().unwrap_or(0.0);
                sum_exp += (s - max_score).exp();
            }
            let lse = max_score + sum_exp.ln();

            // selected child score
            let child_pos = base.saturating_add(child.saturating_mul(child_stride));
            let child_score = values.get(child_pos).copied().unwrap_or(0.0);

            // log probability
            clamp(child_score - lse)
        };

        Op::Value(
            "log_likelihood_table",
            Arity::Exact(dims.len()),
            OpValue::new(data, supplier, modifier),
            operation,
        )
    }
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
}
