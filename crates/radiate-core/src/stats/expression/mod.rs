mod aggregate;
mod builder;
mod logical;
mod named;
mod ops;
mod projection;
mod schedule;
mod select;
mod traits;

pub use named::NamedExpr;
pub use projection::*;
pub use select::SelectExpr;
pub use traits::Evaluate;
pub(crate) use traits::ExprResult;

use aggregate::{AggExpr, BufferExpr};
use logical::When;
use ops::{BinaryExpr, TrinaryExpr, UnaryExpr};
use radiate_utils::{AnyValue, DataType, SmallStr};
use schedule::{EveryState, ScheduleExpr};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod expr_fields {
    use radiate_utils::{DataType, Field};

    pub static STD_DEV: Field = Field::new_const("std_dev", DataType::Float32);
    pub static MEAN: Field = Field::new_const("mean", DataType::Float32);
    pub static MIN: Field = Field::new_const("min", DataType::Float32);
    pub static MAX: Field = Field::new_const("max", DataType::Float32);
    pub static SUM: Field = Field::new_const("sum", DataType::Float32);
    pub static VAR: Field = Field::new_const("var", DataType::Float32);
    pub static SKEW: Field = Field::new_const("skew", DataType::Float32);
    pub static COUNT: Field = Field::new_const("count", DataType::UInt64);
    pub static LAST_VALUE: Field = Field::new_const("last_value", DataType::Float32);
}

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
}

impl<I> Evaluate<I> for Expr
where
    I: ExprProjection,
{
    fn eval<'a>(&'a mut self, input: &I) -> ExprResult<'a> {
        match self {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Selector(selector) => selector.eval(input),
            Expr::Aggregate(child) => child.eval(input),
            Expr::Buffer(child) => child.eval(input),
            Expr::Trinary(child) => child.eval(input),
            Expr::Binary(child) => child.eval(input),
            Expr::Unary(child) => child.eval(input),
            Expr::Schedule(child) => child.eval(input),
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
            Expr::Unary(u) => u.child.reset(),
            Expr::Trinary(t) => {
                t.first.reset();
                t.second.reset();
                t.third.reset();
            }
        }
    }
}

pub mod expr {
    use super::*;
    use super::{expr_fields::LAST_VALUE, select::PathBuilder};

    pub fn lit(value: impl Into<AnyValue<'static>>) -> Expr {
        Expr::Literal(value.into())
    }

    pub fn select(name: impl Into<SmallStr>) -> Expr {
        let small_name = name.into();
        Expr::Selector(SelectExpr::Field(
            AnyValue::StrOwned(small_name.clone().into_string()),
            LAST_VALUE.clone(),
        ))
    }

    pub fn select_with_dtype(name: impl Into<SmallStr>, dtype: DataType) -> Expr {
        let small_name = name.into();
        Expr::Selector(SelectExpr::Field(
            AnyValue::StrOwned(small_name.clone().into_string()),
            LAST_VALUE.clone().with_dtype(dtype),
        ))
    }

    pub fn when(cond: impl Into<Expr>) -> When {
        When::new(cond.into())
    }

    pub fn path(name: impl Into<AnyValue<'static>>) -> PathBuilder {
        PathBuilder::default().key(name.into())
    }

    pub fn nth(n: usize) -> Expr {
        Expr::Selector(SelectExpr::Nth(n))
    }

    pub fn every(interval: usize) -> When {
        When::new(Expr::Schedule(ScheduleExpr::Every(EveryState::new(
            interval,
        ))))
    }

    pub fn element() -> Expr {
        Expr::Selector(SelectExpr::Element)
    }
}

#[cfg(test)]
mod tests {
    use super::{Evaluate, Expr, expr};
    use radiate_utils::{AnyValue, DataType};
    use std::collections::HashMap;

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
        assert!((f32_val(e.eval(&0.0f32).unwrap()) - 3.14).abs() < 1e-6);
    }

    #[test]
    fn lit_ignores_input() {
        let mut e = expr::lit(42.0f32);
        // Same result regardless of what the input is
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 42.0);
        assert_eq!(f32_val(e.eval(&999.0f32).unwrap()), 42.0);
    }

    // ---- Unary ops ----

    #[test]
    fn neg_negates_numeric() {
        let mut e = expr::lit(5.0f32).neg();
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), -5.0);
    }

    #[test]
    fn abs_returns_magnitude() {
        let mut e = expr::lit(-7.0f32).abs();
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 7.0);
    }

    #[test]
    fn not_inverts_bool() {
        let mut t = Expr::Literal(AnyValue::Bool(true)).not();
        let mut f = Expr::Literal(AnyValue::Bool(false)).not();
        assert!(!bool_val(t.eval(&0.0f32).unwrap()));
        assert!(bool_val(f.eval(&0.0f32).unwrap()));
    }

    #[test]
    fn not_on_non_bool_errors() {
        let mut e = expr::lit(1.0f32).not();
        assert!(e.eval(&0.0f32).is_err());
    }

    #[test]
    fn cast_f32_to_i32_truncates() {
        let mut e = expr::lit(3.9f32).cast(DataType::Int32);
        let result = e.eval(&0.0f32).unwrap();
        assert_eq!(result.extract::<i32>(), Some(3));
    }

    // ---- Arithmetic binary ops ----

    #[test]
    fn add_two_literals() {
        let mut e = expr::lit(2.0f32).add(expr::lit(3.0f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 5.0);
    }

    #[test]
    fn sub_two_literals() {
        let mut e = expr::lit(10.0f32).sub(expr::lit(3.0f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 7.0);
    }

    #[test]
    fn mul_two_literals() {
        let mut e = expr::lit(4.0f32).mul(expr::lit(2.5f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 10.0);
    }

    #[test]
    fn div_two_literals() {
        let mut e = expr::lit(9.0f32).div(expr::lit(3.0f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 3.0);
    }

    #[test]
    fn pow_two_literals() {
        let mut e = expr::lit(2.0f32).pow(expr::lit(8.0f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 256.0);
    }

    // ---- Operator overloads ----

    #[test]
    fn add_operator_overload() {
        let mut e = Expr::from(3.0f32) + Expr::from(4.0f32);
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 7.0);
    }

    #[test]
    fn neg_operator_overload() {
        let mut e = -Expr::from(5.0f32);
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), -5.0);
    }

    #[test]
    fn not_operator_overload() {
        let mut e = !Expr::Literal(AnyValue::Bool(true));
        assert!(!bool_val(e.eval(&0.0f32).unwrap()));
    }

    // ---- Comparison ops ----

    #[test]
    fn lt_lte_gt_gte_correct() {
        let five = || expr::lit(5.0f32);
        let ten = || expr::lit(10.0f32);
        let input = &0.0f32;

        assert!(bool_val(five().lt(ten()).eval(input).unwrap()));
        assert!(!bool_val(ten().lt(five()).eval(input).unwrap()));
        assert!(bool_val(five().lte(five()).eval(input).unwrap()));
        assert!(bool_val(ten().gt(five()).eval(input).unwrap()));
        assert!(bool_val(ten().gte(ten()).eval(input).unwrap()));
        assert!(!bool_val(five().gte(ten()).eval(input).unwrap()));
    }

    #[test]
    fn eq_and_ne_correct() {
        let input = &0.0f32;
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
        let input = &0.0f32;
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
        let input = &0.0f32;
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
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 1.0);
    }

    #[test]
    fn when_selects_otherwise_branch_on_false() {
        let mut e = expr::when(Expr::Literal(AnyValue::Bool(false)))
            .then(expr::lit(1.0f32))
            .otherwise(expr::lit(2.0f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 2.0);
    }

    #[test]
    fn when_condition_can_be_a_comparison() {
        let mut e = expr::when(expr::lit(5.0f32).gt(expr::lit(3.0f32)))
            .then(expr::lit(100.0f32))
            .otherwise(expr::lit(0.0f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 100.0);
    }

    // ---- Clamp ----

    #[test]
    fn clamp_below_min_returns_min() {
        let mut e = expr::lit(-5.0f32).clamp(expr::lit(0.0f32), expr::lit(1.0f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 0.0);
    }

    #[test]
    fn clamp_above_max_returns_max() {
        let mut e = expr::lit(10.0f32).clamp(expr::lit(0.0f32), expr::lit(1.0f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 1.0);
    }

    #[test]
    fn clamp_within_range_unchanged() {
        let mut e = expr::lit(0.5f32).clamp(expr::lit(0.0f32), expr::lit(1.0f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 0.5);
    }

    #[test]
    fn clamp_null_input_returns_min() {
        let mut e = Expr::Literal(AnyValue::Null).clamp(expr::lit(0.05f32), expr::lit(2.0f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 0.05);
    }

    #[test]
    fn clamp_nan_input_returns_min() {
        let mut e = expr::lit(f32::NAN).clamp(expr::lit(0.05f32), expr::lit(2.0f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 0.05);
    }

    #[test]
    fn clamp_pos_inf_input_returns_min() {
        let mut e = expr::lit(f32::INFINITY).clamp(expr::lit(0.05f32), expr::lit(2.0f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 0.05);
    }

    #[test]
    fn clamp_neg_inf_input_returns_min() {
        let mut e = expr::lit(f32::NEG_INFINITY).clamp(expr::lit(0.05f32), expr::lit(2.0f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 0.05);
    }

    #[test]
    fn clamp_missing_bounds_errors() {
        let mut e = expr::lit(0.5f32).clamp(Expr::Literal(AnyValue::Null), expr::lit(2.0f32));
        assert!(e.eval(&0.0f32).is_err());
    }

    // ---- or_else (Coalesce) ----

    #[test]
    fn or_else_finite_passes_through() {
        let mut e = expr::lit(3.0f32).or_else(expr::lit(99.0f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 3.0);
    }

    #[test]
    fn or_else_null_falls_back() {
        let mut e = Expr::Literal(AnyValue::Null).or_else(expr::lit(99.0f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 99.0);
    }

    #[test]
    fn or_else_nan_falls_back() {
        let mut e = expr::lit(f32::NAN).or_else(expr::lit(99.0f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 99.0);
    }

    #[test]
    fn or_else_inf_falls_back() {
        let mut e = expr::lit(f32::INFINITY).or_else(expr::lit(99.0f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 99.0);
    }

    #[test]
    fn or_else_neg_inf_falls_back() {
        let mut e = expr::lit(f32::NEG_INFINITY).or_else(expr::lit(99.0f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 99.0);
    }

    #[test]
    fn or_else_chains_through_bad_values() {
        let mut e = Expr::Literal(AnyValue::Null)
            .or_else(expr::lit(f32::NAN))
            .or_else(expr::lit(7.0f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 7.0);
    }

    // ---- min_with / max_with ----

    #[test]
    fn min_with_picks_smaller() {
        let mut e = expr::lit(5.0f32).min_with(expr::lit(3.0f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 3.0);
    }

    #[test]
    fn max_with_picks_larger() {
        let mut e = expr::lit(5.0f32).max_with(expr::lit(8.0f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 8.0);
    }

    #[test]
    fn min_with_nan_on_one_side_returns_other() {
        // f32::min(a, NaN) = a (IEEE 754-2019 minNum semantics)
        let mut e = expr::lit(5.0f32).min_with(expr::lit(f32::NAN));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 5.0);
    }

    #[test]
    fn max_with_nan_on_one_side_returns_other() {
        let mut e = expr::lit(5.0f32).max_with(expr::lit(f32::NAN));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 5.0);
    }

    #[test]
    fn floor_via_max_with_constant() {
        // Common pattern: max_with as a floor without an upper ceiling.
        let mut e = expr::lit(-3.0f32).max_with(expr::lit(0.0f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 0.0);
    }

    // ---- Quantile ----

    #[test]
    fn quantile_empty_returns_zero() {
        let values: Vec<f32> = vec![];
        let mut e = expr::element().quantile(0.5);
        assert_eq!(f32_val(e.eval(&values).unwrap()), 0.0);
    }

    #[test]
    fn quantile_single_value_returns_value() {
        let values = vec![7.0f32];
        let mut e = expr::element().quantile(0.5);
        assert_eq!(f32_val(e.eval(&values).unwrap()), 7.0);
    }

    #[test]
    fn quantile_p50_equals_median() {
        let values = vec![1.0f32, 2.0, 3.0, 4.0, 5.0];
        let mut e = expr::element().quantile(0.5);
        assert_eq!(f32_val(e.eval(&values).unwrap()), 3.0);
    }

    #[test]
    fn quantile_p0_equals_min() {
        let values = vec![3.0f32, 1.0, 2.0, 5.0, 4.0];
        let mut e = expr::element().quantile(0.0);
        assert_eq!(f32_val(e.eval(&values).unwrap()), 1.0);
    }

    #[test]
    fn quantile_p100_equals_max() {
        let values = vec![3.0f32, 1.0, 2.0, 5.0, 4.0];
        let mut e = expr::element().quantile(1.0);
        assert_eq!(f32_val(e.eval(&values).unwrap()), 5.0);
    }

    #[test]
    fn quantile_interpolates_between_ranks() {
        // [1, 2, 3, 4] at q=0.5 → pos = 0.5 * 3 = 1.5 → 0.5*2 + 0.5*3 = 2.5
        let values = vec![1.0f32, 2.0, 3.0, 4.0];
        let mut e = expr::element().quantile(0.5);
        assert!((f32_val(e.eval(&values).unwrap()) - 2.5).abs() < 1e-6);
    }

    #[test]
    fn quantile_filters_nan_and_inf() {
        let values = vec![1.0f32, f32::NAN, 3.0, f32::INFINITY, 5.0];
        let mut e = expr::element().quantile(0.5);
        // After filtering: [1, 3, 5], median = 3
        assert_eq!(f32_val(e.eval(&values).unwrap()), 3.0);
    }

    #[test]
    fn quantile_clamps_q_outside_unit_interval() {
        let values = vec![1.0f32, 2.0, 3.0];
        let mut over = expr::element().quantile(1.5);
        let mut under = expr::element().quantile(-0.5);
        assert_eq!(f32_val(over.eval(&values).unwrap()), 3.0);
        assert_eq!(f32_val(under.eval(&values).unwrap()), 1.0);
    }

    // ---- Expr::reset ----

    #[test]
    fn reset_clears_rolling_buffer() {
        // Rolling mean over a window of 3. Push three values, then reset.
        // After reset the rolling mean should reflect only post-reset pushes.
        let mut e = expr::element().rolling(3).mean();
        assert_eq!(f32_val(e.eval(&10.0f32).unwrap()), 10.0);
        assert_eq!(f32_val(e.eval(&20.0f32).unwrap()), 15.0);
        assert_eq!(f32_val(e.eval(&30.0f32).unwrap()), 20.0);

        e.reset();

        // First post-reset value should stand alone in the buffer.
        assert_eq!(f32_val(e.eval(&100.0f32).unwrap()), 100.0);
    }

    #[test]
    fn reset_clears_schedule_counter() {
        // every(3) fires true on every third call. After two calls + reset,
        // the next call should NOT fire (counter starts fresh).
        let mut e = expr::every(3)
            .then(expr::lit(1.0f32))
            .otherwise(expr::lit(0.0f32));

        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 0.0);
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 0.0);

        e.reset();

        // Two more calls — should still be the "otherwise" branch since the
        // counter restarted at 0.
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 0.0);
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 0.0);
        // Third call from a fresh counter — should fire.
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 1.0);
    }

    #[test]
    fn reset_recurses_into_binary_children() {
        // Rolling means on both sides of an add. After populating and resetting,
        // both sides should be cleared.
        let lhs = expr::element().rolling(2).mean();
        let rhs = expr::lit(1.0f32);
        let mut e = lhs.add(rhs);

        // Populate the left buffer.
        e.eval(&10.0f32).unwrap();
        e.eval(&20.0f32).unwrap();

        e.reset();

        // After reset, lhs starts fresh: 50 → mean=50, plus rhs=1 → 51.
        assert_eq!(f32_val(e.eval(&50.0f32).unwrap()), 51.0);
    }

    #[test]
    fn reset_idempotent_on_leaf() {
        let mut e = expr::lit(42.0f32);
        e.reset();
        e.reset();
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 42.0);
    }

    // ---- Aggregations against Vec<f32> ----

    #[test]
    fn element_returns_all_values() {
        let values = vec![1.0f32, 2.0, 3.0];
        let mut e = expr::element();
        assert!(matches!(e.eval(&values).unwrap(), AnyValue::Vector(_)));
    }

    #[test]
    fn nth_selects_by_index() {
        let values = vec![10.0f32, 20.0, 30.0];
        let mut e = expr::nth(1);
        assert_eq!(f32_val(e.eval(&values).unwrap()), 20.0);
    }

    #[test]
    fn agg_mean_over_vec() {
        let values = vec![1.0f32, 2.0, 3.0, 4.0, 5.0];
        let mut e = expr::element().mean();
        assert_eq!(f32_val(e.eval(&values).unwrap()), 3.0);
    }

    #[test]
    fn agg_sum_over_vec() {
        let values = vec![1.0f32, 2.0, 3.0];
        let mut e = expr::element().sum();
        assert_eq!(f32_val(e.eval(&values).unwrap()), 6.0);
    }

    #[test]
    fn agg_min_max_over_vec() {
        let values = vec![3.0f32, 1.0, 4.0, 1.0, 5.0];
        assert_eq!(f32_val(expr::element().min().eval(&values).unwrap()), 1.0);
        assert_eq!(f32_val(expr::element().max().eval(&values).unwrap()), 5.0);
    }

    #[test]
    fn agg_count_over_vec() {
        let values = vec![1.0f32, 2.0, 3.0, 4.0];
        let mut e = expr::element().count();
        assert_eq!(e.eval(&values).unwrap().extract::<u64>(), Some(4));
    }

    #[test]
    fn agg_first_and_last() {
        let values = vec![10.0f32, 20.0, 30.0];
        assert_eq!(
            f32_val(expr::element().first().eval(&values).unwrap()),
            10.0
        );
        assert_eq!(f32_val(expr::element().last().eval(&values).unwrap()), 30.0);
    }

    #[test]
    fn agg_slope_on_linear_sequence() {
        // y = x with unit steps → slope = 1.0
        let values = vec![0.0f32, 1.0, 2.0, 3.0, 4.0];
        let mut e = expr::element().slope();
        let result = f32_val(e.eval(&values).unwrap());
        assert!((result - 1.0).abs() < 1e-4, "slope was {result}");
    }

    #[test]
    fn agg_unique_removes_duplicates() {
        let values = vec![1.0f32, 2.0, 2.0, 3.0, 1.0];
        let mut e = expr::element().unique();
        let result = e.eval(&values).unwrap();
        if let AnyValue::Vector(items) = result {
            assert_eq!(
                items.len(),
                3,
                "expected 3 unique values, got {}",
                items.len()
            );
        } else {
            panic!("expected Vector, got {result:?}");
        }
    }

    // ---- Rolling window ----

    #[test]
    fn rolling_accumulates_into_slice() {
        let mut e = expr::element().rolling(3);
        e.eval(&vec![1.0f32]).unwrap();
        e.eval(&vec![2.0f32]).unwrap();
        let result = e.eval(&vec![3.0f32]).unwrap();
        // After 3 pushes the window is full — result is a Slice of the buffered values
        assert!(
            result.is_nested(),
            "expected nested slice after window fill"
        );
    }

    // ---- Schedule: every(n) ----

    #[test]
    fn every_fires_on_nth_call_then_resets() {
        let mut e = expr::every(3)
            .then(Expr::Literal(AnyValue::Bool(true)))
            .otherwise(Expr::Literal(AnyValue::Bool(false)));

        assert!(!bool_val(e.eval(&0.0f32).unwrap())); // tick 1
        assert!(!bool_val(e.eval(&0.0f32).unwrap())); // tick 2
        assert!(bool_val(e.eval(&0.0f32).unwrap())); // tick 3 — fires
        assert!(!bool_val(e.eval(&0.0f32).unwrap())); // tick 1 again
        assert!(!bool_val(e.eval(&0.0f32).unwrap())); // tick 2 again
        assert!(bool_val(e.eval(&0.0f32).unwrap())); // tick 3 — fires again
    }

    // ---- HashMap projection ----

    #[test]
    fn path_select_from_hashmap() {
        let mut map = HashMap::new();
        map.insert("accuracy".to_string(), 0.95f32);
        let mut e: Expr = expr::path("accuracy").into();
        assert!((f32_val(e.eval(&map).unwrap()) - 0.95).abs() < 1e-6);
    }

    #[test]
    fn missing_key_in_hashmap_errors() {
        let map: HashMap<String, f32> = HashMap::new();
        let mut e: Expr = expr::path("missing").into();
        assert!(e.eval(&map).is_err());
    }

    // ---- Composition ----

    #[test]
    fn composed_expr_add_then_compare() {
        // (2 + 3) > 4 → true
        let mut e = expr::lit(2.0f32)
            .add(expr::lit(3.0f32))
            .gt(expr::lit(4.0f32));
        assert!(bool_val(e.eval(&0.0f32).unwrap()));
    }

    #[test]
    fn composed_expr_clamp_then_scale() {
        // clamp(-5, 0, 1) * 10 → 0.0
        let mut e = expr::lit(-5.0f32)
            .clamp(expr::lit(0.0f32), expr::lit(1.0f32))
            .mul(expr::lit(10.0f32));
        assert_eq!(f32_val(e.eval(&0.0f32).unwrap()), 0.0);
    }
}
