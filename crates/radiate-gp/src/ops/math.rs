use super::Op;
use crate::{
    Arity,
    ops::{Param, op_names},
};
use radiate_core::random_provider;
use smallvec::SmallVec;

use super::OpFloat;
use crate::{Eval, EvalInto};

// Every math op is defined once, as a function of plain values below. An op's *shape* (how it
// reads its inputs) then turns that one function into both of its forms:
//
//   shape     scalar form (one sample)        column form (every sample)
//   ───────   ─────────────────────────────   ──────────────────────────────────────
//   unary     f(x[0])                         out[r] = f(x0[r])
//   binary    f(x[0], x[1])                   out[r] = f(x0[r], x1[r])
//   summed    f(x[0] + x[1] + ...)            out = x0 + x1 + ...; out[r] = f(out[r])
//   folded    clamp(seed ∘ x[0] ∘ x[1] ...)   out = seed; out[r] = out[r] ∘ xi[r]; clamp
//
// The scalar form goes into the op's `Op::Fn`, and the column form into `column_kernel`. Both
// call the same function in the same order, so they stay bit-identical.

/// Clamp a value to the range [-MAX_VALUE, MAX_VALUE]. Without this, values can quickly
/// become too large or too small to be useful.
#[inline]
fn clamp<F: OpFloat>(value: F) -> F {
    if value.is_nan() {
        return F::ZERO;
    }
    value.clamp(-F::MAX_VALUE, F::MAX_VALUE)
}

/// Aggregate a slice of values by summing them. There usually aren't too many inputs, so we
/// can use an if statement to handle a few of the common cases - vals with a len <= 5.
#[inline]
fn aggregate<F: OpFloat>(vals: &[F]) -> F {
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

// Binary ops.

#[inline(always)]
fn add<F: OpFloat>(a: F, b: F) -> F {
    clamp(a + b)
}

#[inline(always)]
fn sub<F: OpFloat>(a: F, b: F) -> F {
    clamp(a - b)
}

#[inline(always)]
fn mul<F: OpFloat>(a: F, b: F) -> F {
    clamp(a * b)
}

#[inline(always)]
fn div<F: OpFloat>(a: F, b: F) -> F {
    if b.abs() < F::EPS {
        F::ONE
    } else {
        clamp(a / b)
    }
}

#[inline(always)]
fn pow<F: OpFloat>(a: F, b: F) -> F {
    clamp(a.powf(b))
}

#[inline(always)]
fn weight<F: OpFloat>(x: F, w: F) -> F {
    clamp(x * w)
}

// Unary ops.

#[inline(always)]
fn neg<F: OpFloat>(x: F) -> F {
    clamp(-x)
}

#[inline(always)]
fn abs<F: OpFloat>(x: F) -> F {
    clamp(x.abs())
}

#[inline(always)]
fn ceil<F: OpFloat>(x: F) -> F {
    clamp(x.ceil())
}

#[inline(always)]
fn floor<F: OpFloat>(x: F) -> F {
    clamp(x.floor())
}

#[inline(always)]
fn reciprocal<F: OpFloat>(x: F) -> F {
    if x.abs() < F::EPS {
        F::ONE
    } else {
        clamp(F::ONE / x)
    }
}

#[inline(always)]
fn sign<F: OpFloat>(x: F) -> F {
    if x > F::ZERO {
        F::ONE
    } else if x < F::ZERO {
        -F::ONE
    } else {
        F::ZERO
    }
}

#[inline(always)]
fn sqrt<F: OpFloat>(x: F) -> F {
    clamp(x.sqrt())
}

#[inline(always)]
fn exp<F: OpFloat>(x: F) -> F {
    clamp(x.exp())
}

#[inline(always)]
fn log<F: OpFloat>(x: F) -> F {
    if x > F::ZERO { clamp(x.ln()) } else { F::ZERO }
}

#[inline(always)]
fn sin<F: OpFloat>(x: F) -> F {
    clamp(x.sin())
}

#[inline(always)]
fn cos<F: OpFloat>(x: F) -> F {
    clamp(x.cos())
}

#[inline(always)]
fn tan<F: OpFloat>(x: F) -> F {
    clamp(x.tan())
}

// Applied to the sum of the inputs (the summed shape): the activations, plus sum, gaussian and
// tooth. `clamp` itself is sum and linear.

#[inline(always)]
fn sigmoid<F: OpFloat>(total: F) -> F {
    clamp(F::ONE / (F::ONE + (-total).exp()))
}

#[inline(always)]
fn tanh<F: OpFloat>(total: F) -> F {
    clamp(total.tanh())
}

#[inline(always)]
fn relu<F: OpFloat>(total: F) -> F {
    clamp(total.max(F::ZERO))
}

#[inline(always)]
fn leaky_relu<F: OpFloat>(total: F) -> F {
    let x = clamp(total);
    if x > F::ZERO { x } else { clamp(F::HALF * x) }
}

#[inline(always)]
fn elu<F: OpFloat>(total: F) -> F {
    let x = clamp(total);
    if x > F::ZERO {
        x
    } else {
        clamp(F::HALF * (x.exp() - F::ONE))
    }
}

#[inline(always)]
fn mish<F: OpFloat>(total: F) -> F {
    let x = clamp(total);
    clamp(x * (x.exp().ln_1p().tanh()))
}

#[inline(always)]
fn swish<F: OpFloat>(total: F) -> F {
    let x = clamp(total);
    clamp(x / (F::ONE + (-x).exp()))
}

#[inline(always)]
fn softplus<F: OpFloat>(total: F) -> F {
    let x = clamp(total);
    let threshold = F::from(20.0).unwrap();
    if x > threshold {
        x
    } else if x < -threshold {
        x.exp()
    } else {
        (F::ONE + x.exp()).ln()
    }
}

#[inline(always)]
fn gaussian<F: OpFloat>(total: F) -> F {
    let x = clamp(total);
    clamp((-x * x).exp())
}

#[inline(always)]
fn tooth<F: OpFloat>(total: F) -> F {
    let x = clamp(total);
    clamp(x % F::ONE)
}

// Folded ops: a running value combined with each input in turn.

#[inline(always)]
fn product<F: OpFloat>(acc: F, x: F) -> F {
    acc * x
}

#[inline(always)]
fn difference<F: OpFloat>(acc: F, x: F) -> F {
    acc - x
}

#[inline]
fn logsumexp<F: OpFloat>(xs: &[F]) -> F {
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

// Shapes, scalar form: one sample's inputs in, one value out.

#[inline(always)]
fn unary<F: OpFloat>(vals: &[F], f: impl Fn(F) -> F) -> F {
    f(vals[0])
}

#[inline(always)]
fn binary<F: OpFloat>(vals: &[F], f: impl Fn(F, F) -> F) -> F {
    f(vals[0], vals[1])
}

#[inline(always)]
fn summed<F: OpFloat>(vals: &[F], f: impl Fn(F) -> F) -> F {
    f(aggregate(vals))
}

/// `seed` starts the fold; `None` starts it from the first input instead (zero with no inputs).
#[inline(always)]
fn folded<F: OpFloat>(vals: &[F], seed: Option<F>, step: impl Fn(F, F) -> F) -> F {
    let (seed, rest) = match (seed, vals.split_first()) {
        (Some(seed), _) => (seed, vals),
        (None, Some((first, rest))) => (*first, rest),
        (None, None) => (F::ZERO, vals),
    };
    clamp(rest.iter().copied().fold(seed, step))
}

pub enum AggregateOperations {
    Sum,
    Prod,
    Diff,
    Max,
    Min,
    LogSumExp,
    Gaussian,
    Tooth,
}

impl AggregateOperations {
    pub fn apply<F: OpFloat>(&self, inputs: &[F]) -> F {
        match self {
            AggregateOperations::Sum => summed(inputs, clamp),
            // x1 - x2 - ... - xn: the first input minus every other input.
            AggregateOperations::Diff => folded(inputs, None, difference),
            AggregateOperations::Prod => folded(inputs, Some(F::ONE), product),
            AggregateOperations::Max => folded(inputs, Some(-F::MAX_VALUE), F::max),
            AggregateOperations::Min => folded(inputs, Some(F::MAX_VALUE), F::min),
            AggregateOperations::LogSumExp => clamp(logsumexp(inputs)),
            AggregateOperations::Gaussian => summed(inputs, gaussian),
            AggregateOperations::Tooth => summed(inputs, tooth),
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
    pub fn apply<F: OpFloat>(&self, inputs: &[F]) -> F {
        match self {
            ActivationOperation::Sigmoid => summed(inputs, sigmoid),
            ActivationOperation::Tanh => summed(inputs, tanh),
            ActivationOperation::ReLU => summed(inputs, relu),
            ActivationOperation::LeakyReLU => summed(inputs, leaky_relu),
            ActivationOperation::ELU => summed(inputs, elu),
            ActivationOperation::Linear => summed(inputs, clamp),
            ActivationOperation::Mish => summed(inputs, mish),
            ActivationOperation::Swish => summed(inputs, swish),
            ActivationOperation::Softplus => summed(inputs, softplus),
        }
    }
}

impl<F: OpFloat> Op<F> {
    pub fn weight() -> Self {
        Self::weight_with(random_provider::range(-F::ONE..F::ONE))
    }

    pub fn dot() -> Self {
        Self::dot_with((F::ONE, F::ONE))
    }

    pub fn weight_with(value: F) -> Self {
        let supplier = || random_provider::range(-F::ONE..F::ONE);

        let operation = |inputs: &[F], w: &F| weight(inputs[0], *w);

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

    pub fn dot_with((a, b): (F, F)) -> Self {
        let supplier = || {
            let a = random_provider::range(-F::ONE..F::ONE);
            let b = random_provider::range(-F::ONE..F::ONE);
            (a, b)
        };

        let operation =
            |inputs: &[F], weight: &(F, F)| clamp(inputs[0] * weight.0 + inputs[1] * weight.1);

        let modifier = |current: &mut (F, F)| {
            let side = random_provider::bool(0.5);
            let diff = random_provider::range(-F::ONE..F::ONE) * F::TENTH;
            if side {
                current.0 = clamp(current.0 + diff);
            } else {
                current.1 = clamp(current.1 + diff);
            }
        };

        Op::Pair(
            op_names::DOT,
            2.into(),
            Param::new((clamp(a), clamp(b)), supplier, modifier),
            operation,
        )
    }

    /// Ephemeral Random Constant (ERC).
    pub fn erc() -> Self {
        let supplier = || random_provider::range(-F::ONE..F::ONE);

        let modifier = |current: &mut F| {
            let diff = random_provider::range(-F::ONE..F::ONE) * F::TENTH;
            *current = clamp(*current + diff);
        };

        Op::Value(
            op_names::ERC,
            0.into(),
            Param::new(clamp(supplier()), supplier, modifier),
            |_: &[F], w: &F| *w,
        )
    }

    pub fn sign() -> Self {
        Op::Fn(op_names::SIGN, 1.into(), |inputs: &[F]| unary(inputs, sign))
    }

    pub fn reciprocal() -> Self {
        Op::Fn(op_names::RECIPROCAL, 1.into(), |inputs: &[F]| {
            unary(inputs, reciprocal)
        })
    }

    pub fn add() -> Self {
        Op::Fn(op_names::ADD, 2.into(), |inputs: &[F]| binary(inputs, add))
    }

    pub fn sub() -> Self {
        Op::Fn(op_names::SUB, 2.into(), |inputs: &[F]| binary(inputs, sub))
    }

    pub fn mul() -> Self {
        Op::Fn(op_names::MUL, 2.into(), |inputs: &[F]| binary(inputs, mul))
    }

    pub fn div() -> Self {
        Op::Fn(op_names::DIV, 2.into(), |inputs: &[F]| binary(inputs, div))
    }

    pub fn neg() -> Self {
        Op::Fn(op_names::NEG, 1.into(), |inputs: &[F]| unary(inputs, neg))
    }

    pub fn pow() -> Self {
        Op::Fn(op_names::POW, 2.into(), |inputs: &[F]| binary(inputs, pow))
    }

    pub fn sqrt() -> Self {
        Op::Fn(op_names::SQRT, 1.into(), |inputs: &[F]| unary(inputs, sqrt))
    }

    pub fn abs() -> Self {
        Op::Fn(op_names::ABS, 1.into(), |inputs: &[F]| unary(inputs, abs))
    }

    pub fn exp() -> Self {
        Op::Fn(op_names::EXP, 1.into(), |inputs: &[F]| unary(inputs, exp))
    }

    pub fn log() -> Self {
        Op::Fn(op_names::LOG, 1.into(), |inputs: &[F]| unary(inputs, log))
    }

    pub fn sin() -> Self {
        Op::Fn(op_names::SIN, 1.into(), |inputs: &[F]| unary(inputs, sin))
    }

    pub fn cos() -> Self {
        Op::Fn(op_names::COS, 1.into(), |inputs: &[F]| unary(inputs, cos))
    }

    pub fn tan() -> Self {
        Op::Fn(op_names::TAN, 1.into(), |inputs: &[F]| unary(inputs, tan))
    }

    pub fn ceil() -> Self {
        Op::Fn(op_names::CEIL, 1.into(), |inputs: &[F]| unary(inputs, ceil))
    }

    pub fn floor() -> Self {
        Op::Fn(op_names::FLOOR, 1.into(), |inputs: &[F]| {
            unary(inputs, floor)
        })
    }

    pub fn tooth() -> Self {
        Op::Fn(op_names::TOOTH, Arity::Any, |inputs: &[F]| {
            AggregateOperations::Tooth.apply(inputs)
        })
    }

    pub fn gaussian() -> Self {
        Op::Fn(op_names::GAUSSIAN, Arity::Any, |inputs: &[F]| {
            AggregateOperations::Gaussian.apply(inputs)
        })
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
pub fn math_ops<F: OpFloat>() -> Vec<Op<F>> {
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
        Op::tooth(),
        Op::reciprocal(),
        Op::gaussian(),
        Op::sign(),
    ]
}

/// Get a list of all the activation operations.
pub fn activation_ops<F: OpFloat>() -> Vec<Op<F>> {
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

pub fn edge_ops<F: OpFloat>() -> Vec<Op<F>> {
    vec![Op::weight(), Op::identity()]
}

pub fn input_ops<F: OpFloat>() -> Vec<Op<F>> {
    vec![Op::erc()]
}

/// Get a list of all the operations.
pub fn all_ops<F: OpFloat>() -> Vec<Op<F>> {
    math_ops()
        .into_iter()
        .chain(activation_ops())
        .chain(edge_ops())
        .chain(input_ops())
        .collect()
}

// Shapes, column form: `inputs[i]` is the i-th input's column (one value per sample) and `out`
// receives one value per sample. Each matches its scalar form above bit-for-bit, and returns
// `false` when there are too few inputs, so the caller falls back to the scalar op (which
// panics on the same inputs, just like row evaluation).

#[inline(always)]
fn unary_columns<F: OpFloat>(inputs: &[&[F]], out: &mut [F], f: impl Fn(F) -> F) -> bool {
    let [x, ..] = inputs else {
        return false;
    };

    for (o, &x) in out.iter_mut().zip(*x) {
        *o = f(x);
    }
    true
}

#[inline(always)]
fn binary_columns<F: OpFloat>(inputs: &[&[F]], out: &mut [F], f: impl Fn(F, F) -> F) -> bool {
    let [a, b, ..] = inputs else {
        return false;
    };

    for ((o, &x), &y) in out.iter_mut().zip(*a).zip(*b) {
        *o = f(x, y);
    }
    true
}

/// Sums the input columns, then applies `f`. The sum matches [aggregate] bit-for-bit:
/// 1..=5 inputs sum left to right starting from the first input, more fold from zero.
#[inline(always)]
fn summed_columns<F: OpFloat>(inputs: &[&[F]], out: &mut [F], f: impl Fn(F) -> F) -> bool {
    match inputs.split_first() {
        None => out.fill(F::ZERO),
        Some((first, rest)) => {
            if inputs.len() <= 5 {
                out.copy_from_slice(first);
            } else {
                out.fill(F::ZERO);
                for (o, &x) in out.iter_mut().zip(*first) {
                    *o = *o + x;
                }
            }

            for col in rest {
                for (o, &x) in out.iter_mut().zip(*col) {
                    *o = *o + x;
                }
            }
        }
    }

    for o in out.iter_mut() {
        *o = f(*o);
    }
    true
}

#[inline(always)]
fn folded_columns<F: OpFloat>(
    inputs: &[&[F]],
    out: &mut [F],
    seed: Option<F>,
    step: impl Fn(F, F) -> F,
) -> bool {
    let rest = match (seed, inputs.split_first()) {
        (Some(seed), _) => {
            out.fill(seed);
            inputs
        }
        (None, Some((first, rest))) => {
            out.copy_from_slice(first);
            rest
        }
        (None, None) => {
            out.fill(F::ZERO);
            inputs
        }
    };

    for col in rest {
        for (o, &x) in out.iter_mut().zip(*col) {
            *o = step(*o, x);
        }
    }

    for o in out.iter_mut() {
        *o = clamp(*o);
    }
    true
}

/// Column kernel for `op`: computes it for every sample at once, where `inputs[i]` is the
/// column of the op's i-th input and `out` receives one value per sample. Returns `false` when
/// the op has no kernel (or too few inputs); [Op]'s column [EvalInto] then falls back to calling
/// the op once per sample.
///
/// Ops are identified by name, the same convention deserialization uses (see `ops/serde.rs`),
/// so a custom op that reuses a built-in name gets the built-in kernel. Each arm uses the same
/// shape and function as the op's constructor, so results are bit-identical.
fn process_op_vectorized<F: OpFloat>(op: &Op<F>, inputs: &[&[F]], out: &mut [F]) -> bool {
    match op {
        // Same rules as the scalar ops: a var reads its input at `index`, a const ignores inputs.
        Op::Var(_, index, _) => match inputs.get(*index) {
            Some(x) => {
                out.copy_from_slice(x);
                true
            }
            None => false,
        },
        Op::Const(_, v) => {
            out.fill(*v);
            true
        }
        Op::Value(name, _, param, _) => match *name {
            op_names::WEIGHT => {
                let w = *param.data();
                unary_columns(inputs, out, |x| weight(x, w))
            }
            op_names::ERC => {
                let w = *param.data();
                out.fill(w);
                true
            }
            _ => false,
        },
        Op::Fn(name, _, _) => match *name {
            op_names::IDENTITY => match inputs.first() {
                Some(x) => {
                    out.copy_from_slice(x);
                    true
                }
                None => false,
            },

            op_names::ADD => binary_columns(inputs, out, add),
            op_names::SUB => binary_columns(inputs, out, sub),
            op_names::MUL => binary_columns(inputs, out, mul),
            op_names::DIV => binary_columns(inputs, out, div),
            op_names::POW => binary_columns(inputs, out, pow),

            op_names::NEG => unary_columns(inputs, out, neg),
            op_names::ABS => unary_columns(inputs, out, abs),
            op_names::CEIL => unary_columns(inputs, out, ceil),
            op_names::FLOOR => unary_columns(inputs, out, floor),
            op_names::RECIPROCAL => unary_columns(inputs, out, reciprocal),
            op_names::SIGN => unary_columns(inputs, out, sign),
            op_names::SQRT => unary_columns(inputs, out, sqrt),
            op_names::EXP => unary_columns(inputs, out, exp),
            op_names::LOG => unary_columns(inputs, out, log),
            op_names::SIN => unary_columns(inputs, out, sin),
            op_names::COS => unary_columns(inputs, out, cos),
            op_names::TAN => unary_columns(inputs, out, tan),

            op_names::SUM | op_names::LINEAR => summed_columns(inputs, out, clamp),
            op_names::SIGMOID => summed_columns(inputs, out, sigmoid),
            op_names::TANH => summed_columns(inputs, out, tanh),
            op_names::RELU => summed_columns(inputs, out, relu),
            op_names::LEAKY_RELU => summed_columns(inputs, out, leaky_relu),
            op_names::ELU => summed_columns(inputs, out, elu),
            op_names::MISH => summed_columns(inputs, out, mish),
            op_names::SWISH => summed_columns(inputs, out, swish),
            op_names::SOFTPLUS => summed_columns(inputs, out, softplus),
            op_names::GAUSSIAN => summed_columns(inputs, out, gaussian),
            op_names::TOOTH => summed_columns(inputs, out, tooth),

            op_names::DIFF => folded_columns(inputs, out, None, difference),
            op_names::PROD => folded_columns(inputs, out, Some(F::ONE), product),
            op_names::MAX => folded_columns(inputs, out, Some(-F::MAX_VALUE), F::max),
            op_names::MIN => folded_columns(inputs, out, Some(F::MAX_VALUE), F::min),

            _ => false,
        },
        _ => false,
    }
}

/// Inputs gathered on the stack, per sample, up to this fan-in in the per-sample fallback.
const STACK_INPUTS: usize = 8;

/// Column evaluation of an [Op]: `inputs[i]` is the column of the op's i-th input (one value per
/// sample) and `buffer` receives the op's value for every sample. The column counterpart of
/// `Eval<[F], F> for Op<F>`, with the same meaning for every op, e.g. a var reads input
/// `index` and a const ignores its inputs.
///
/// Uses the op's column kernel when it has one ([column_kernel]); otherwise calls the scalar op
/// once per sample, gathering that sample's inputs out of the columns:
///
/// ```text
///   sample r:   row = [ inputs[0][r], inputs[1][r], ... ]  →  buffer[r] = self.eval(row)
/// ```
impl<'a, F: OpFloat> EvalInto<[&'a [F]], [F]> for Op<F> {
    fn eval_into(&self, inputs: &[&'a [F]], buffer: &mut [F]) {
        if process_op_vectorized(self, inputs, buffer) {
            return;
        }

        let mut row = SmallVec::<[F; STACK_INPUTS]>::from_elem(F::ZERO, inputs.len());
        for (r, out) in buffer.iter_mut().enumerate() {
            for (value, column) in row.iter_mut().zip(inputs) {
                *value = column[r];
            }
            *out = self.eval(&row[..inputs.len()]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_kernels_match_scalar_ops() {
        let ops = all_ops::<f32>().into_iter().chain([
            Op::identity(),
            Op::weight_with(-0.7),
            Op::weight_with(0.3),
        ]);

        // Values around zero plus extremes, so clamp, -0.0 and the div EPS branch get hit.
        let values = [
            -3.0, -1.0, -0.5, -0.0, 0.0, 1e-9, 0.25, 0.5, 1.0, 2.5, 7.0, -40.0, 1e12, -1e12,
        ];
        let rows = 64;
        let mut kernels = 0;

        for op in ops {
            for fan_in in 1..=7 {
                let cols: Vec<Vec<f32>> = (0..fan_in)
                    .map(|i| {
                        (0..rows)
                            .map(|r| values[(r * (i + 3) + i) % values.len()])
                            .collect()
                    })
                    .collect();
                let inputs: Vec<&[f32]> = cols.iter().map(|c| c.as_slice()).collect();

                let mut out = vec![0.0; rows];
                if !process_op_vectorized(&op, &inputs, &mut out) {
                    continue;
                }
                kernels += 1;

                for r in 0..rows {
                    let row: Vec<f32> = cols.iter().map(|c| c[r]).collect();
                    let want = op.eval(&row);
                    assert_eq!(
                        out[r].to_bits(),
                        want.to_bits(),
                        "{} with {fan_in} inputs, row {r}: column {} vs scalar {}",
                        op.name(),
                        out[r],
                        want
                    );
                }
            }
        }

        assert!(kernels > 0);
    }
    use crate::{Eval, ops::math};

    #[inline]
    fn approx<F: OpFloat>(a: F, b: F, eps: F) -> bool {
        (a - b).abs() <= eps
    }

    #[test]
    fn clamp_behaves_as_specified_f32() {
        assert_eq!(math::clamp(f32::NAN), 0.0_f32);
        assert_eq!(math::clamp(1e20_f32), f32::MAX_VALUE);
        assert_eq!(math::clamp(-1e20_f32), -f32::MAX_VALUE);
        assert_eq!(math::clamp(123.456_f32), 123.456_f32);
    }

    #[test]
    fn clamp_behaves_as_specified_f64() {
        assert_eq!(math::clamp(f64::NAN), 0.0_f64);
        assert_eq!(math::clamp(1e20_f64), f64::MAX_VALUE);
        assert_eq!(math::clamp(-1e20_f64), -f64::MAX_VALUE);
        assert_eq!(math::clamp(123.456_f64), 123.456_f64);
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
        assert_eq!(AggregateOperations::Diff.apply(&xs), -5.0);
        assert_eq!(AggregateOperations::Diff.apply(&[5.0_f32, 3.0]), 2.0);
        assert_eq!(AggregateOperations::Diff.apply(&[7.0_f32]), 7.0);
        assert_eq!(AggregateOperations::Diff.apply::<f32>(&[]), 0.0);
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
