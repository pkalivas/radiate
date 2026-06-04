mod aggregate;
mod builder;
mod compile;
mod logical;
mod ops;
mod query;
mod schedule;
mod select;
mod stagnation;
mod traits;

pub use query::MetricQuery;
pub use select::{MetricField, MetricKind, SelectExpr};
pub use traits::Evaluate;
pub(crate) use traits::ExprResult;

use aggregate::{AggExpr, BufferExpr};
use logical::When;
use ops::{BinaryExpr, TrinaryExpr, UnaryExpr};
use radiate_utils::{AnyValue, SmallStr};
use schedule::{EveryState, ScheduleExpr};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use stagnation::StagnationExpr;

// /// Multiplicative integrator: `out_t = clamp(out_{t-1} * factor_t, lo, hi)`,
// /// seeded with `seed` on the first eval (and after `reset`). This is the
// /// self-anchored controller `expr::track` lowers to — it carries its own
// /// previous output, so no external "previous threshold" metric is needed.
// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// #[derive(Clone, Debug, PartialEq)]
// pub struct FeedbackExpr {
//     pub(super) factor: Box<Expr>,
//     pub(super) seed: f32,
//     pub(super) lo: f32,
//     pub(super) hi: f32,
//     pub(super) state: Option<f32>, // prev output; None until first eval
// }

// impl FeedbackExpr {
//     pub fn new(factor: Expr, seed: f32, lo: f32, hi: f32) -> Self {
//         Self {
//             factor: Box::new(factor),
//             seed,
//             lo,
//             hi,
//             state: None,
//         }
//     }
//     pub(super) fn reset(&mut self) {
//         self.state = None;
//         self.factor.reset();
//     }
// }

// impl Evaluate for FeedbackExpr {
//     fn eval<'a>(&'a mut self, metrics: &MetricSet) -> ExprResult<'a> {
//         let prev = self.state.unwrap_or(self.seed);
//         // Missing/non-finite factor (e.g. gen 0 before the metric exists) → hold.
//         let factor = self
//             .factor
//             .eval(metrics)
//             .ok()
//             .and_then(|v| v.extract::<f32>())
//             .filter(|f| f.is_finite())
//             .unwrap_or(1.0);
//         let out = (prev * factor).clamp(self.lo, self.hi);
//         self.state = Some(out);
//         Ok(AnyValue::Float32(out))
//     }
// }

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Literal(AnyValue<'static>),
    Selector(SelectExpr),
    Aggregate(AggExpr),
    Buffer(BufferExpr),
    Schedule(ScheduleExpr),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Trinary(TrinaryExpr),
    Stagnation(StagnationExpr),
}

impl Evaluate for Expr {
    fn eval<'a>(&'a mut self, metrics: &crate::MetricSet) -> ExprResult<'a> {
        match self {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Selector(selector) => selector.eval(metrics),
            Expr::Aggregate(child) => child.eval(metrics),
            Expr::Buffer(child) => child.eval(metrics),
            Expr::Trinary(child) => child.eval(metrics),
            Expr::Binary(child) => child.eval(metrics),
            Expr::Unary(child) => child.eval(metrics),
            Expr::Schedule(child) => child.eval(metrics),
            Expr::Stagnation(child) => child.eval(metrics),
        }
    }
}

impl Expr {
    /// Recursively clears state in stateful operators: rolling-window buffers
    /// in `Aggregate`/`Buffer` nodes and counters in `Schedule::Every`. Children
    /// of binary/unary/trinary nodes are also visited. Leaf nodes (literals,
    /// selectors) are unaffected.
    ///
    /// Use after an engine restart or whenever the controller should "forget"
    /// accumulated history.
    pub fn reset(&mut self) {
        match self {
            Expr::Literal(_) | Expr::Selector(_) => {}
            Expr::Aggregate(a) => a.reset(),
            Expr::Buffer(b) => b.reset(),
            Expr::Schedule(ScheduleExpr::Every(s)) => s.reset(),
            Expr::Binary(b) => {
                b.lhs.reset();
                b.rhs.reset();
            }
            Expr::Unary(u) => {
                u.child.reset();
                if let ops::UnaryOp::Quantile(q) = &mut u.op {
                    q.clear();
                }
            }
            Expr::Trinary(t) => {
                t.first.reset();
                t.second.reset();
                t.third.reset();
            }
            Expr::Stagnation(s) => s.reset(),
        }
    }
}

pub mod expr {
    use super::*;

    pub fn lit(value: impl Into<AnyValue<'static>>) -> Expr {
        Expr::Literal(value.into())
    }

    pub fn select(name: impl Into<SmallStr>) -> Expr {
        Expr::Selector(SelectExpr::new(name))
    }

    pub fn when(cond: impl Into<Expr>) -> When {
        When::new(cond.into())
    }

    pub fn every(interval: usize) -> When {
        When::new(Expr::Schedule(ScheduleExpr::Every(EveryState::new(
            interval,
        ))))
    }

    /// Relative error from a target: `(x - target) / target`.
    /// Fuses into a single Affine node.
    pub fn error_from(metric: impl Into<SmallStr>, target: f32) -> Expr {
        select(metric).error_from(target)
    }

    /// Convergence detector: true when `|first - last|` over the rolling window
    /// drops below `epsilon`. Two independent rolling buffers, one for First,
    /// one for Last.
    pub fn is_converged(metric: impl Into<SmallStr>, window: usize, epsilon: f32) -> Expr {
        let m = metric.into();
        let first = select(m.clone()).rolling(window).first();
        let last = select(m).rolling(window).last();
        first.sub(last).abs().lt(lit(epsilon))
    }

    /// Counts consecutive evaluations during which `metric.last_value` has
    /// stayed within `epsilon` of the value last considered an improvement.
    /// Resets on any change exceeding `epsilon`.
    pub fn stagnation(metric: impl Into<SmallStr>, epsilon: f32) -> Expr {
        Expr::Stagnation(StagnationExpr::new(metric, epsilon))
    }

    /// True when `stagnation(metric, epsilon) >= patience`.
    pub fn is_stagnant(metric: impl Into<SmallStr>, patience: u32, epsilon: f32) -> Expr {
        stagnation(metric, epsilon).gte(lit(patience as f32))
    }

    /// PI-style control signal for a metric tracking toward a target. Returns
    /// `1 + gain * (rolling_mean(metric) - target) / target`. Multiply this
    /// by an anchor (e.g. observed magnitude of the controlled variable) and
    /// clamp to produce a final rate.
    pub fn pi_signal(metric: impl Into<SmallStr>, target: f32, gain: f32, window: usize) -> Expr {
        select(metric)
            .rolling(window)
            .mean()
            .error_from(target)
            .affine(gain, 1.0)
            .compile()
    }

    /// Streaming median (P²) of a metric. Constant memory; sees all observations
    /// since construction. For a windowed exact median, use
    /// `select(metric).rolling(N).quantile(0.5)`.
    pub fn p50(metric: impl Into<SmallStr>) -> Expr {
        select(metric).quantile_stream(0.5)
    }

    /// Streaming 95th-percentile (P²) of a metric.
    pub fn p95(metric: impl Into<SmallStr>) -> Expr {
        select(metric).quantile_stream(0.95)
    }

    /// Streaming 99th-percentile (P²) of a metric.
    pub fn p99(metric: impl Into<SmallStr>) -> Expr {
        select(metric).quantile_stream(0.99)
    }
}

#[cfg(test)]
mod tests {
    use super::ops::UnaryOp;
    use super::{Evaluate, Expr, expr};
    use crate::MetricSet;
    use radiate_utils::{AnyValue, DataType};

    fn is_fused_affine(e: &Expr) -> bool {
        matches!(e, Expr::Unary(u) if matches!(u.op, UnaryOp::Affine { .. }))
    }

    fn metrics() -> MetricSet {
        MetricSet::default()
    }

    fn f32_val(v: AnyValue<'_>) -> f32 {
        v.extract::<f32>().expect("expected f32")
    }

    fn bool_val(v: AnyValue<'_>) -> bool {
        match v {
            AnyValue::Bool(b) => b,
            other => panic!("expected bool, got {other:?}"),
        }
    }

    // ---- Literals ----

    #[test]
    fn lit_evaluates_to_its_value() {
        let mut e = expr::lit(3.14f32);
        assert!((f32_val(e.eval(&metrics()).unwrap()) - 3.14).abs() < 1e-6);
    }

    #[test]
    fn lit_ignores_input() {
        let mut e = expr::lit(42.0f32);
        // Same result regardless of what the input is
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 42.0);
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 42.0);
    }

    // ---- Unary ops ----

    #[test]
    fn neg_negates_numeric() {
        let mut e = expr::lit(5.0f32).neg();
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), -5.0);
    }

    #[test]
    fn abs_returns_magnitude() {
        let mut e = expr::lit(-7.0f32).abs();
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 7.0);
    }

    #[test]
    fn not_inverts_bool() {
        let mut t = Expr::Literal(AnyValue::Bool(true)).not();
        let mut f = Expr::Literal(AnyValue::Bool(false)).not();
        assert!(!bool_val(t.eval(&metrics()).unwrap()));
        assert!(bool_val(f.eval(&metrics()).unwrap()));
    }

    #[test]
    fn not_on_non_bool_errors() {
        let mut e = expr::lit(1.0f32).not();
        assert!(e.eval(&metrics()).is_err());
    }

    #[test]
    fn cast_f32_to_i32_truncates() {
        let mut e = expr::lit(3.9f32).cast(DataType::Int32);
        let result = e.eval(&metrics()).unwrap();
        assert_eq!(result.extract::<i32>(), Some(3));
    }

    // ---- Arithmetic binary ops ----

    #[test]
    fn add_two_literals() {
        let mut e = expr::lit(2.0f32).add(expr::lit(3.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 5.0);
    }

    #[test]
    fn sub_two_literals() {
        let mut e = expr::lit(10.0f32).sub(expr::lit(3.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 7.0);
    }

    #[test]
    fn mul_two_literals() {
        let mut e = expr::lit(4.0f32).mul(expr::lit(2.5f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 10.0);
    }

    #[test]
    fn div_two_literals() {
        let mut e = expr::lit(9.0f32).div(expr::lit(3.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 3.0);
    }

    #[test]
    fn pow_two_literals() {
        let mut e = expr::lit(2.0f32).pow(expr::lit(8.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 256.0);
    }

    // ---- Operator overloads ----

    #[test]
    fn add_operator_overload() {
        let mut e = Expr::from(3.0f32) + Expr::from(4.0f32);
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 7.0);
    }

    #[test]
    fn neg_operator_overload() {
        let mut e = -Expr::from(5.0f32);
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), -5.0);
    }

    #[test]
    fn not_operator_overload() {
        let mut e = !Expr::Literal(AnyValue::Bool(true));
        assert!(!bool_val(e.eval(&metrics()).unwrap()));
    }

    // ---- Comparison ops ----

    #[test]
    fn lt_lte_gt_gte_correct() {
        let five = || expr::lit(5.0f32);
        let ten = || expr::lit(10.0f32);
        let input = &metrics();

        assert!(bool_val(five().lt(ten()).eval(input).unwrap()));
        assert!(!bool_val(ten().lt(five()).eval(input).unwrap()));
        assert!(bool_val(five().lte(five()).eval(input).unwrap()));
        assert!(bool_val(ten().gt(five()).eval(input).unwrap()));
        assert!(bool_val(ten().gte(ten()).eval(input).unwrap()));
        assert!(!bool_val(five().gte(ten()).eval(input).unwrap()));
    }

    #[test]
    fn eq_and_ne_correct() {
        let input = &metrics();
        assert!(bool_val(
            expr::lit(5.0f32).eq(expr::lit(5.0f32)).eval(input).unwrap()
        ));
        assert!(!bool_val(
            expr::lit(5.0f32).eq(expr::lit(6.0f32)).eval(input).unwrap()
        ));
        assert!(bool_val(
            expr::lit(5.0f32).ne(expr::lit(6.0f32)).eval(input).unwrap()
        ));
    }

    #[test]
    fn between_is_inclusive_on_both_ends() {
        let input = &metrics();
        let range = || (expr::lit(1.0f32), expr::lit(10.0f32));

        let (lo, hi) = range();
        assert!(bool_val(
            expr::lit(5.0f32).between(lo, hi).eval(input).unwrap()
        ));

        let (lo, hi) = range();
        assert!(bool_val(
            expr::lit(1.0f32).between(lo, hi).eval(input).unwrap()
        ));

        let (lo, hi) = range();
        assert!(bool_val(
            expr::lit(10.0f32).between(lo, hi).eval(input).unwrap()
        ));

        let (lo, hi) = range();
        assert!(!bool_val(
            expr::lit(0.0f32).between(lo, hi).eval(input).unwrap()
        ));
    }

    // ---- Logical ops ----

    #[test]
    fn and_or_short_circuit_values() {
        let input = &metrics();
        let t = || Expr::Literal(AnyValue::Bool(true));
        let f = || Expr::Literal(AnyValue::Bool(false));

        assert!(!bool_val(t().and(f()).eval(input).unwrap()));
        assert!(bool_val(t().and(t()).eval(input).unwrap()));
        assert!(bool_val(f().or(t()).eval(input).unwrap()));
        assert!(!bool_val(f().or(f()).eval(input).unwrap()));
    }

    // ---- When / then / otherwise ----

    #[test]
    fn when_selects_then_branch_on_true() {
        let mut e = expr::when(Expr::Literal(AnyValue::Bool(true)))
            .then(expr::lit(1.0f32))
            .otherwise(expr::lit(2.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 1.0);
    }

    #[test]
    fn when_selects_otherwise_branch_on_false() {
        let mut e = expr::when(Expr::Literal(AnyValue::Bool(false)))
            .then(expr::lit(1.0f32))
            .otherwise(expr::lit(2.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 2.0);
    }

    #[test]
    fn when_condition_can_be_a_comparison() {
        let mut e = expr::when(expr::lit(5.0f32).gt(expr::lit(3.0f32)))
            .then(expr::lit(100.0f32))
            .otherwise(expr::lit(0.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 100.0);
    }

    // ---- Clamp ----

    #[test]
    fn clamp_below_min_returns_min() {
        let mut e = expr::lit(-5.0f32).clamp(expr::lit(0.0f32), expr::lit(1.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 0.0);
    }

    #[test]
    fn clamp_above_max_returns_max() {
        let mut e = expr::lit(10.0f32).clamp(expr::lit(0.0f32), expr::lit(1.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 1.0);
    }

    #[test]
    fn clamp_within_range_unchanged() {
        let mut e = expr::lit(0.5f32).clamp(expr::lit(0.0f32), expr::lit(1.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 0.5);
    }

    #[test]
    fn clamp_null_input_returns_min() {
        let mut e = Expr::Literal(AnyValue::Null).clamp(expr::lit(0.05f32), expr::lit(2.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 0.05);
    }

    #[test]
    fn clamp_nan_input_returns_min() {
        let mut e = expr::lit(f32::NAN).clamp(expr::lit(0.05f32), expr::lit(2.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 0.05);
    }

    #[test]
    fn clamp_pos_inf_input_returns_min() {
        let mut e = expr::lit(f32::INFINITY).clamp(expr::lit(0.05f32), expr::lit(2.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 0.05);
    }

    #[test]
    fn clamp_neg_inf_input_returns_min() {
        let mut e = expr::lit(f32::NEG_INFINITY).clamp(expr::lit(0.05f32), expr::lit(2.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 0.05);
    }

    #[test]
    fn clamp_missing_bounds_errors() {
        let mut e = expr::lit(0.5f32).clamp(Expr::Literal(AnyValue::Null), expr::lit(2.0f32));
        assert!(e.eval(&metrics()).is_err());
    }

    // ---- or_else (Coalesce) ----

    #[test]
    fn or_else_finite_passes_through() {
        let mut e = expr::lit(3.0f32).or_else(expr::lit(99.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 3.0);
    }

    #[test]
    fn or_else_null_falls_back() {
        let mut e = Expr::Literal(AnyValue::Null).or_else(expr::lit(99.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 99.0);
    }

    #[test]
    fn or_else_nan_falls_back() {
        let mut e = expr::lit(f32::NAN).or_else(expr::lit(99.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 99.0);
    }

    #[test]
    fn or_else_inf_falls_back() {
        let mut e = expr::lit(f32::INFINITY).or_else(expr::lit(99.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 99.0);
    }

    #[test]
    fn or_else_neg_inf_falls_back() {
        let mut e = expr::lit(f32::NEG_INFINITY).or_else(expr::lit(99.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 99.0);
    }

    #[test]
    fn or_else_chains_through_bad_values() {
        let mut e = Expr::Literal(AnyValue::Null)
            .or_else(expr::lit(f32::NAN))
            .or_else(expr::lit(7.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 7.0);
    }

    // ---- min_with / max_with ----

    #[test]
    fn min_with_picks_smaller() {
        let mut e = expr::lit(5.0f32).min_with(expr::lit(3.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 3.0);
    }

    #[test]
    fn max_with_picks_larger() {
        let mut e = expr::lit(5.0f32).max_with(expr::lit(8.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 8.0);
    }

    #[test]
    fn min_with_nan_on_one_side_returns_other() {
        // f32::min(a, NaN) = a (IEEE 754-2019 minNum semantics)
        let mut e = expr::lit(5.0f32).min_with(expr::lit(f32::NAN));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 5.0);
    }

    #[test]
    fn max_with_nan_on_one_side_returns_other() {
        let mut e = expr::lit(5.0f32).max_with(expr::lit(f32::NAN));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 5.0);
    }

    #[test]
    fn floor_via_max_with_constant() {
        // Common pattern: max_with as a floor without an upper ceiling.
        let mut e = expr::lit(-3.0f32).max_with(expr::lit(0.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 0.0);
    }

    // ---- Expr::reset ----

    #[test]
    fn reset_clears_schedule_counter() {
        // every(3) fires true on every third call. After two calls + reset,
        // the next call should NOT fire (counter starts fresh).
        let mut e = expr::every(3)
            .then(expr::lit(1.0f32))
            .otherwise(expr::lit(0.0f32));

        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 0.0);
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 0.0);

        e.reset();

        // Two more calls — should still be the "otherwise" branch since the
        // counter restarted at 0.
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 0.0);
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 0.0);
        // Third call from a fresh counter — should fire.
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 1.0);
    }

    #[test]
    fn reset_idempotent_on_leaf() {
        let mut e = expr::lit(42.0f32);
        e.reset();
        e.reset();
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 42.0);
    }

    // ---- Schedule: every(n) ----

    #[test]
    fn every_fires_on_nth_call_then_resets() {
        let mut e = expr::every(3)
            .then(Expr::Literal(AnyValue::Bool(true)))
            .otherwise(Expr::Literal(AnyValue::Bool(false)));

        assert!(!bool_val(e.eval(&metrics()).unwrap())); // tick 1
        assert!(!bool_val(e.eval(&metrics()).unwrap())); // tick 2
        assert!(bool_val(e.eval(&metrics()).unwrap())); // tick 3 — fires
        assert!(!bool_val(e.eval(&metrics()).unwrap())); // tick 1 again
        assert!(!bool_val(e.eval(&metrics()).unwrap())); // tick 2 again
        assert!(bool_val(e.eval(&metrics()).unwrap())); // tick 3 — fires again
    }

    // ---- Pre-built composers ----

    fn metrics_with(name: &str, value: f32) -> MetricSet {
        let mut ms = MetricSet::new();
        ms.upsert(name, value);
        ms
    }

    #[test]
    fn error_from_method_collapses_to_affine() {
        // (x - 10) / 10 == x * 0.1 - 1
        let e = expr::lit(15.0f32).error_from(10.0);
        assert!(is_fused_affine(&e), "expected fused Affine, got {e:?}");
        let mut e = e;
        assert!((f32_val(e.eval(&metrics()).unwrap()) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn error_from_function_reads_metric() {
        let ms = metrics_with("foo", 12.0);
        let mut e = expr::error_from("foo", 10.0);
        // (12 - 10) / 10 = 0.2
        assert!((f32_val(e.eval(&ms).unwrap()) - 0.2).abs() < 1e-6);
    }

    #[test]
    fn pi_signal_produces_expected_shape() {
        // Window=1 mean = current value. error_from(10) on x=12 = 0.2.
        // 1 + 0.5 * 0.2 = 1.1
        let mut ms = metrics_with("count", 12.0);
        let mut e = expr::pi_signal("count", 10.0, 0.5, 1);
        // Need to push two values to fill window=1 buffer — first call seeds, second returns.
        let _ = e.eval(&ms);
        ms.upsert("count", 12.0);
        let v = f32_val(e.eval(&ms).unwrap());
        assert!((v - 1.1).abs() < 1e-4, "got {v}");
    }

    // ---- Streaming quantile (P²) ----

    #[test]
    fn quantile_stream_returns_first_sample_until_buffer_fills() {
        let mut e = expr::p50("foo");
        let ms = metrics_with("foo", 5.0);
        // First sample seeds the estimator; with one sample p50 == that sample.
        assert!((f32_val(e.eval(&ms).unwrap()) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn quantile_stream_null_when_metric_missing() {
        let mut e = expr::p95("missing");
        let ms = MetricSet::new();
        assert!(matches!(e.eval(&ms).unwrap(), AnyValue::Null));
    }

    #[test]
    fn quantile_stream_converges_on_uniform_sequence() {
        let mut e = expr::p50("foo");
        let mut ms = MetricSet::new();
        for i in 1..=200 {
            ms.upsert("foo", i as f32);
            let _ = e.eval(&ms);
        }
        // True median is 100.5; P² is approximate but should be close.
        let v = f32_val(e.eval(&ms).unwrap());
        assert!(
            (v - 100.5).abs() < 3.0,
            "p50 estimate {v} far from true median 100.5"
        );
    }

    #[test]
    fn quantile_stream_p95_approximates_high_tail() {
        let mut e = expr::p95("foo");
        let mut ms = MetricSet::new();
        for i in 1..=1000 {
            ms.upsert("foo", i as f32);
            let _ = e.eval(&ms);
        }
        let v = f32_val(e.eval(&ms).unwrap());
        assert!((v - 950.0).abs() < 20.0, "p95 estimate {v} far from 950");
    }

    #[test]
    fn quantile_stream_reset_clears_estimator() {
        let mut e = expr::p50("foo");
        let mut ms = MetricSet::new();
        for i in 1..=50 {
            ms.upsert("foo", i as f32);
            let _ = e.eval(&ms);
        }
        e.reset();
        // After reset, first eval should produce just-seeded estimator value.
        ms.upsert("foo", 7.0);
        let v = f32_val(e.eval(&ms).unwrap());
        assert!((v - 7.0).abs() < 1e-6, "got {v}");
    }

    #[test]
    fn quantile_stream_composes_with_arbitrary_child() {
        // Stream p50 of a *literal* — exercises the "any child" composition.
        let mut e = expr::lit(42.0f32).quantile_stream(0.5);
        let ms = metrics();
        let _ = e.eval(&ms);
        let _ = e.eval(&ms);
        // After multiple identical samples, p50 == constant.
        assert!((f32_val(e.eval(&ms).unwrap()) - 42.0).abs() < 1e-6);
    }

    // ---- Stagnation ----

    #[test]
    fn stagnation_increments_when_value_unchanged() {
        let ms = metrics_with("score", 1.0);
        let mut e = expr::stagnation("score", 0.001);

        assert_eq!(f32_val(e.eval(&ms).unwrap()), 0.0); // seed
        assert_eq!(f32_val(e.eval(&ms).unwrap()), 1.0);
        assert_eq!(f32_val(e.eval(&ms).unwrap()), 2.0);
    }

    #[test]
    fn stagnation_resets_on_large_change() {
        let mut ms = metrics_with("score", 1.0);
        let mut e = expr::stagnation("score", 0.001);

        let _ = e.eval(&ms);
        let _ = e.eval(&ms); // count = 1

        ms.upsert("score", 5.0); // big change > epsilon
        assert_eq!(f32_val(e.eval(&ms).unwrap()), 0.0);
        assert_eq!(f32_val(e.eval(&ms).unwrap()), 1.0);
    }

    #[test]
    fn stagnation_tolerates_tiny_noise() {
        let mut ms = metrics_with("score", 1.0);
        let mut e = expr::stagnation("score", 0.01);

        let _ = e.eval(&ms);
        ms.upsert("score", 1.005); // within epsilon
        assert_eq!(f32_val(e.eval(&ms).unwrap()), 1.0);
        ms.upsert("score", 1.008);
        assert_eq!(f32_val(e.eval(&ms).unwrap()), 2.0);
    }

    #[test]
    fn stagnation_returns_null_when_metric_missing() {
        let ms = MetricSet::new();
        let mut e = expr::stagnation("missing", 0.001);
        assert!(matches!(e.eval(&ms).unwrap(), AnyValue::Null));
    }

    #[test]
    fn is_stagnant_fires_at_patience_threshold() {
        let ms = metrics_with("score", 1.0);
        let mut e = expr::is_stagnant("score", 3, 0.001);

        assert!(!bool_val(e.eval(&ms).unwrap())); // count=0
        assert!(!bool_val(e.eval(&ms).unwrap())); // count=1
        assert!(!bool_val(e.eval(&ms).unwrap())); // count=2
        assert!(bool_val(e.eval(&ms).unwrap())); // count=3, fires
    }

    #[test]
    fn stagnation_reset_clears_state() {
        let ms = metrics_with("score", 1.0);
        let mut e = expr::stagnation("score", 0.001);

        let _ = e.eval(&ms);
        let _ = e.eval(&ms);
        let _ = e.eval(&ms); // count = 2

        e.reset();
        assert_eq!(f32_val(e.eval(&ms).unwrap()), 0.0); // fresh seed
    }

    #[test]
    fn is_converged_fires_when_window_is_flat() {
        let ms = metrics_with("score", 1.0);
        let mut e = expr::is_converged("score", 3, 0.01);

        // Buffers seed up to size 3.
        let _ = e.eval(&ms);
        let _ = e.eval(&ms);
        // Third eval: both first and last buffers full, both should hold ~1.0.
        assert!(bool_val(e.eval(&ms).unwrap()));
    }

    #[test]
    fn is_converged_does_not_fire_when_window_drifts() {
        let mut ms = metrics_with("score", 1.0);
        let mut e = expr::is_converged("score", 3, 0.01);

        let _ = e.eval(&ms);
        ms.upsert("score", 2.0);
        let _ = e.eval(&ms);
        ms.upsert("score", 3.0);
        // first=1.0, last=3.0, diff=2.0 > epsilon
        assert!(!bool_val(e.eval(&ms).unwrap()));
    }

    // ---- compile() ----

    #[test]
    fn compile_folds_pure_literal_subtree() {
        let e = expr::lit(2.0f32).add(expr::lit(3.0f32)).compile();
        assert!(matches!(e, Expr::Literal(_)));
        let mut e = e;
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 5.0);
    }

    #[test]
    fn compile_wraps_metric_plus_lit_as_affine() {
        let e = expr::select("foo").add(expr::lit(3.0f32)).compile();
        assert!(is_fused_affine(&e), "expected Affine, got {e:?}");
    }

    #[test]
    fn compile_collapses_controller_chain_to_single_affine() {
        // Replicates the species-threshold count_error chain:
        //   (x - target) / target * GAIN + 1.0
        //   = x * (GAIN / target) + (1 - GAIN)
        let target = 10.0f32;
        let gain = 0.999f32;
        let e = expr::select("count.species")
            .sub(expr::lit(target))
            .div(expr::lit(target))
            .mul(expr::lit(gain))
            .add(expr::lit(1.0f32))
            .compile();

        assert!(is_fused_affine(&e), "expected single Affine, got {e:?}");
    }

    #[test]
    fn compile_is_idempotent() {
        let e = expr::select("foo")
            .sub(expr::lit(1.0f32))
            .mul(expr::lit(2.0f32))
            .add(expr::lit(3.0f32));
        let once = e.clone().compile();
        let twice = once.clone().compile();
        assert_eq!(format!("{:?}", once), format!("{:?}", twice));
    }

    // ---- Affine ----

    #[test]
    fn affine_computes_scale_x_plus_bias() {
        let mut e = expr::lit(5.0f32).affine(2.0, 3.0);
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 13.0);
    }

    #[test]
    fn affine_propagates_null_on_non_finite() {
        let mut nan_e = expr::lit(f32::NAN).affine(2.0, 3.0);
        let mut inf_e = expr::lit(f32::INFINITY).affine(2.0, 3.0);
        let mut null_e = Expr::Literal(AnyValue::Null).affine(2.0, 3.0);
        assert!(matches!(nan_e.eval(&metrics()).unwrap(), AnyValue::Null));
        assert!(matches!(inf_e.eval(&metrics()).unwrap(), AnyValue::Null));
        assert!(matches!(null_e.eval(&metrics()).unwrap(), AnyValue::Null));
    }

    #[test]
    fn affine_chain_collapses_to_single_node() {
        // affine(affine(x, 2, 3), 4, 5) = 4*(2x + 3) + 5 = 8x + 17
        let e = expr::lit(1.0f32).affine(2.0, 3.0).affine(4.0, 5.0);
        assert!(is_fused_affine(&e), "expected single Affine, got {e:?}");
        let mut e = e;
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 25.0); // 8 + 17
    }

    // ---- Composition ----

    #[test]
    fn composed_expr_add_then_compare() {
        // (2 + 3) > 4 → true
        let mut e = expr::lit(2.0f32)
            .add(expr::lit(3.0f32))
            .gt(expr::lit(4.0f32));
        assert!(bool_val(e.eval(&metrics()).unwrap()));
    }

    #[test]
    fn composed_expr_clamp_then_scale() {
        // clamp(-5, 0, 1) * 10 → 0.0
        let mut e = expr::lit(-5.0f32)
            .clamp(expr::lit(0.0f32), expr::lit(1.0f32))
            .mul(expr::lit(10.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 0.0);
    }
}
