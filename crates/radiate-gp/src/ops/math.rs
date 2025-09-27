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

/// Implementations of the [MathOperation] enum. These are the basic math operations.
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
            MathOperation::Log => clamp(if inputs[0] > ZERO {
                inputs[0].ln()
            } else {
                ZERO
            }),
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

/// Implementations of the [ActivationOperation] enum. These are the basic activation functions used
/// in neural networks. However, they are particularly useful in this context because they can
/// accept any number of inputs. Thus, they act as reducers or aggregates and are a key part of
/// being able to define complex 'Graph' and 'Tree' structures.
impl ActivationOperation {
    #[inline]
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
            name: "w",
            arity: 1.into(),
            value: clamp(value),
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
    fn aggregate_len_cases() {
        // Current semantics: len==0 -> ZERO; len==1 -> returns value (does NOT apply f)
        let f = |x: f32| x * 2.0;

        let empty: [f32; 0] = [];
        assert_eq!(super::aggregate(&empty, f), ZERO);

        let single = [5.0];
        assert_eq!(
            super::aggregate(&single, f),
            5.0,
            "len==1 path returns the value directly"
        );

        let two = [2.0, 3.0];
        assert_eq!(super::aggregate(&two, f), 10.0);

        let three = [1.0, 2.0, 3.0];
        assert_eq!(super::aggregate(&three, f), 12.0);

        let four = [1.0, 2.0, 3.0, 4.0];
        assert_eq!(super::aggregate(&four, f), 20.0);

        let five = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(super::aggregate(&five, f), 30.0);

        let many = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        assert_eq!(super::aggregate(&many, f), 42.0);
    }

    #[test]
    fn math_add_sub_mul_div() {
        let a = [8.0, 2.0];
        assert_eq!(MathOperation::Add.apply(&a), 10.0);
        assert_eq!(MathOperation::Sub.apply(&a), 6.0);
        assert_eq!(MathOperation::Mul.apply(&a), 16.0);
        assert_eq!(MathOperation::Div.apply(&a), 4.0);
    }

    #[test]
    fn math_div_near_zero_clamps_large_quotient() {
        let xs = [10.0, 1e-12_f32];
        let y = MathOperation::Div.apply(&xs);
        assert_eq!(
            y, MAX_VALUE,
            "huge quotient should clamp to MAX_VALUE with current code"
        );
    }

    #[test]
    fn math_sum_prod_diff_pow_sqrt_abs() {
        let xs = [2.0, 3.0, 4.0];
        assert_eq!(MathOperation::Sum.apply(&xs), 9.0);
        assert_eq!(MathOperation::Prod.apply(&xs), 24.0);
        // Diff is left fold from ZERO: (((0-2)-3)-4) = -9
        assert_eq!(MathOperation::Diff.apply(&xs), -9.0);

        let p = MathOperation::Pow.apply(&[3.0, 2.0]);
        assert_eq!(p, 9.0);

        assert_eq!(MathOperation::Sqrt.apply(&[9.0]), 3.0);
        assert_eq!(MathOperation::Abs.apply(&[-5.0]), 5.0);
    }

    #[test]
    fn math_exp_log_trig_rounding() {
        let e = MathOperation::Exp.apply(&[1.0]);
        assert!(approx(e, f32::consts::E, 1e-5), "exp(1) ~= e");

        // log on <=0 becomes NaN, then clamp -> 0.0
        assert_eq!(MathOperation::Log.apply(&[0.0]), 0.0);
        assert_eq!(MathOperation::Log.apply(&[-1.0]), 0.0);

        let s = MathOperation::Sin.apply(&[f32::consts::PI / 2.0]);
        assert!(approx(s, 1.0, 1e-5));

        let c = MathOperation::Cos.apply(&[0.0]);
        assert!(approx(c, 1.0, 1e-5));

        let t = MathOperation::Tan.apply(&[0.0]);
        assert!(approx(t, 0.0, 1e-6));

        assert_eq!(MathOperation::Ceil.apply(&[1.2]), 2.0);
        assert_eq!(MathOperation::Floor.apply(&[1.8]), 1.0);
    }

    #[test]
    fn math_max_min_variadic_including_empty_behavior() {
        let xs = [1.5, -2.0, 7.25, 3.0];
        let mx = MathOperation::Max.apply(&xs);
        let mn = MathOperation::Min.apply(&xs);
        assert_eq!(mx, 7.25);
        assert_eq!(mn, -2.0);

        let empty: [f32; 0] = [];
        assert_eq!(MathOperation::Max.apply(&empty), MIN_VALUE);
        assert_eq!(MathOperation::Min.apply(&empty), MAX_VALUE);
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
