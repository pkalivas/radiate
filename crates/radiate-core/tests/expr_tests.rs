#[cfg(test)]
mod test {

    use radiate_core::{AnyValue, MetricSet};
    pub use radiate_expr::*;
    use std::time::Duration;

    fn f32_of(value: AnyValue<'_>) -> f32 {
        value.extract::<f32>().unwrap()
    }

    fn bool_of(value: AnyValue<'_>) -> bool {
        if let AnyValue::Bool(b) = value {
            b
        } else {
            false
        }
    }

    fn u64_of(value: AnyValue<'_>) -> u64 {
        value.extract::<u64>().unwrap()
    }

    #[test]
    fn test_rolling_mean() {
        let mut expr = Expr::select("accuracy").rolling(3).mean();
        let mut metrics = MetricSet::default();

        metrics.upsert("accuracy", 1.0);
        assert!((f32_of(expr.eval(&metrics).unwrap()) - 1.0).abs() < 1e-6);

        metrics.upsert("accuracy", 2.0);
        assert!((f32_of(expr.eval(&metrics).unwrap()) - 1.5).abs() < 1e-6);

        metrics.upsert("accuracy", 3.0);
        assert!((f32_of(expr.eval(&metrics).unwrap()) - 2.0).abs() < 1e-6);

        metrics.upsert("accuracy", 4.0);
        assert!((f32_of(expr.eval(&metrics).unwrap()) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_rolling_sum() {
        let mut expr = Expr::select("accuracy").rolling(3).sum();
        let mut metrics = MetricSet::default();

        metrics.upsert("accuracy", 1.0);
        assert!((f32_of(expr.eval(&metrics).unwrap()) - 1.0).abs() < 1e-6);

        metrics.upsert("accuracy", 2.0);
        assert!((f32_of(expr.eval(&metrics).unwrap()) - 3.0).abs() < 1e-6);

        metrics.upsert("accuracy", 3.0);
        assert!((f32_of(expr.eval(&metrics).unwrap()) - 6.0).abs() < 1e-6);

        metrics.upsert("accuracy", 4.0);
        assert!((f32_of(expr.eval(&metrics).unwrap()) - 9.0).abs() < 1e-6);
    }

    #[test]
    fn test_rolling_min_and_max() {
        let mut min_expr = Expr::select("accuracy").rolling(4).min();
        let mut max_expr = Expr::select("accuracy").rolling(4).max();
        let mut metrics = MetricSet::default();

        for value in [3.0, 1.0, 4.0, 2.0] {
            metrics.upsert("accuracy", value);
            min_expr.eval(&metrics).unwrap();
            max_expr.eval(&metrics).unwrap();
        }

        assert!((f32_of(min_expr.eval(&metrics).unwrap()) - 1.0).abs() < 1e-6);
        assert!((f32_of(max_expr.eval(&metrics).unwrap()) - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_rolling_count() {
        let mut expr = Expr::select("accuracy").rolling(3).count();
        let mut metrics = MetricSet::default();

        metrics.upsert("accuracy", 10.0);
        assert_eq!(u64_of(expr.eval(&metrics).unwrap()), 1);

        metrics.upsert("accuracy", 11.0);
        assert_eq!(u64_of(expr.eval(&metrics).unwrap()), 2);

        metrics.upsert("accuracy", 12.0);
        assert_eq!(u64_of(expr.eval(&metrics).unwrap()), 3);

        metrics.upsert("accuracy", 13.0);
        assert_eq!(u64_of(expr.eval(&metrics).unwrap()), 3);
    }

    #[test]
    fn test_rolling_n_unique() {
        let mut expr = Expr::select("accuracy").rolling(5).unique().count();
        let mut metrics = MetricSet::default();

        metrics.upsert("accuracy", 1.0);
        assert_eq!(u64_of(expr.eval(&metrics).unwrap()), 1);

        metrics.upsert("accuracy", 2.0);
        assert_eq!(u64_of(expr.eval(&metrics).unwrap()), 2);

        metrics.upsert("accuracy", 2.0);
        assert_eq!(u64_of(expr.eval(&metrics).unwrap()), 2);

        metrics.upsert("accuracy", 3.0);
        assert_eq!(u64_of(expr.eval(&metrics).unwrap()), 3);

        metrics.upsert("accuracy", 1.0);
        assert_eq!(u64_of(expr.eval(&metrics).unwrap()), 3);
    }

    #[test]
    fn test_lt_comparison_true_and_false() {
        let mut expr = Expr::select("accuracy").lt(Expr::select("loss"));
        let mut metrics = MetricSet::default();

        metrics.upsert("accuracy", 0.8);
        metrics.upsert("loss", 1.2);
        assert_eq!(bool_of(expr.eval(&metrics).unwrap()), true);

        metrics.upsert("accuracy", 2.0);
        metrics.upsert("loss", 1.2);
        assert_eq!(bool_of(expr.eval(&metrics).unwrap()), false);
    }

    #[test]
    fn test_gte_comparison() {
        let mut expr = Expr::select("accuracy").gte(Expr::select("target"));
        let mut metrics = MetricSet::default();

        metrics.upsert("accuracy", 0.95);
        metrics.upsert("target", 0.90);
        assert!(bool_of(expr.eval(&metrics).unwrap()));

        metrics.upsert("accuracy", 0.85);
        metrics.upsert("target", 0.90);
        assert!(!bool_of(expr.eval(&metrics).unwrap()));
    }

    #[test]
    fn test_eq_comparison_uses_epsilon() {
        let mut expr = Expr::select("a").eq(Expr::select("b"));
        let mut metrics = MetricSet::default();

        metrics.upsert("a", 1.0f32);
        metrics.upsert("b", 1.0f32);
        assert!(bool_of(expr.eval(&metrics).unwrap()));
    }

    #[test]
    fn test_ne_comparison() {
        let mut expr = Expr::select("a").ne(Expr::select("b"));
        let mut metrics = MetricSet::default();

        metrics.upsert("a", 1.0f32);
        metrics.upsert("b", 2.0f32);
        assert!(bool_of(expr.eval(&metrics).unwrap()));

        metrics.upsert("a", 5.0f32);
        metrics.upsert("b", 5.0f32);
        assert!(!bool_of(expr.eval(&metrics).unwrap()));
    }

    #[test]
    fn test_metric_projection_uses_metricset_property_mean() {
        let mut metrics = MetricSet::default();

        metrics.upsert("accuracy", 1.0);
        metrics.upsert("accuracy", 2.0);
        metrics.upsert("accuracy", 3.0);

        // let result = metrics
        //     .project(
        //         &"accuracy".into(),
        //         &Field::new("mean".into(), DataType::Float32),
        //     )
        //     .unwrap();
        // assert!((f32_of(result) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_between_inclusive() {
        let mut expr = Expr::select("x").between(1.0, 3.0);
        let mut metrics = MetricSet::default();

        metrics.upsert("x", 1.0);
        assert!(bool_of(expr.eval(&metrics).unwrap()));

        metrics.upsert("x", 2.0);
        assert!(bool_of(expr.eval(&metrics).unwrap()));

        metrics.upsert("x", 3.0);
        assert!(bool_of(expr.eval(&metrics).unwrap()));

        metrics.upsert("x", 0.99);
        assert!(!bool_of(expr.eval(&metrics).unwrap()));

        metrics.upsert("x", 3.01);
        assert!(!bool_of(expr.eval(&metrics).unwrap()));
    }

    #[test]
    fn test_add_expr() {
        let mut expr = Expr::select("a").add(Expr::select("b"));
        let mut metrics = MetricSet::default();

        metrics.upsert("a", 2.0);
        metrics.upsert("b", 3.5);

        assert!((f32_of(expr.eval(&metrics).unwrap()) - 5.5).abs() < 1e-6);
    }

    #[test]
    fn test_sub_expr() {
        let mut expr = Expr::select("a").sub(Expr::select("b"));
        let mut metrics = MetricSet::default();

        metrics.upsert("a", 5.0);
        metrics.upsert("b", 1.5);

        assert!((f32_of(expr.eval(&metrics).unwrap()) - 3.5).abs() < 1e-6);
    }

    #[test]
    fn test_mul_expr() {
        let mut expr = Expr::select("a").mul(2.5);
        let mut metrics = MetricSet::default();

        metrics.upsert("a", 4.0);

        assert!((f32_of(expr.eval(&metrics).unwrap()) - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_div_expr() {
        let mut expr = Expr::select("a").div(Expr::select("b"));
        let mut metrics = MetricSet::default();

        metrics.upsert("a", 9.0);
        metrics.upsert("b", 3.0);

        assert!((f32_of(expr.eval(&metrics).unwrap()) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_div_by_zero_returns_null() {
        let mut expr = Expr::select("a").div(Expr::select("b"));
        let mut metrics = MetricSet::default();

        metrics.upsert("a", 9.0);
        metrics.upsert("b", 0.0);

        assert_eq!(expr.eval(&metrics).unwrap(), AnyValue::Null);
    }

    #[test]
    fn test_neg_expr() {
        let mut expr = Expr::select("a").neg();
        let mut metrics = MetricSet::default();

        metrics.upsert("a", 4.0);

        assert!((f32_of(expr.eval(&metrics).unwrap()) + 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_abs_expr() {
        let mut expr = Expr::select("a").sub(10.0).abs();
        let mut metrics = MetricSet::default();

        metrics.upsert("a", 4.0);
        assert!((f32_of(expr.eval(&metrics).unwrap()) - 6.0).abs() < 1e-6);

        metrics.upsert("a", 14.0);
        assert!((f32_of(expr.eval(&metrics).unwrap()) - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_expr() {
        let mut expr = Expr::select("a").clamp(0.1, 0.5);
        let mut metrics = MetricSet::default();

        metrics.upsert("a", 0.05);
        assert!((f32_of(expr.eval(&metrics).unwrap()) - 0.1).abs() < 1e-6);

        metrics.upsert("a", 0.25);
        assert!((f32_of(expr.eval(&metrics).unwrap()) - 0.25).abs() < 1e-6);

        metrics.upsert("a", 0.9);
        assert!((f32_of(expr.eval(&metrics).unwrap()) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_duration_expr() {
        let mut expr = Expr::select("time").time().rolling(10).min();
        let mut metrics = MetricSet::default();

        println!("{:#?}", expr);

        metrics.upsert("time", Duration::from_secs(5));
        expr.eval(&metrics).unwrap();
        metrics.upsert("time", Duration::from_secs(3));
        expr.eval(&metrics).unwrap();
        metrics.upsert("time", Duration::from_secs(8));
        let result = expr.eval(&metrics);

        assert_eq!(result.unwrap(), AnyValue::Duration(Duration::from_secs(3)));
    }

    #[test]
    fn test_every_expr() {
        let mut expr = Expr::every(3)
            .then(Expr::select("accuracy").mean())
            .otherwise(0.0);

        let mut metrics = MetricSet::default();
        let inputs = [1.0, 2.0, 3.0, 4.0, 5.0];

        for (i, &value) in inputs.iter().enumerate() {
            metrics.upsert("accuracy", value);
            let result = expr.eval(&metrics);
            println!("Input: {value}, Output: {result:?}");

            if i % 3 == 2 {
                let expected_mean = inputs[i - 2..=i].iter().sum::<f32>() / 3.0;
                assert!((f32_of(result.unwrap()) - expected_mean).abs() < 1e-6);
            } else {
                assert!((f32_of(result.unwrap()) - 0.0).abs() < 1e-6);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use radiate_core::MetricSet;
    use radiate_expr::{Evaluate, Expr, ExprKind};
    use radiate_utils::{AnyValue, DataType};

    // fn is_fused_affine(e: &Expr) -> bool {
    //     matches!(e, Expr::Unary(u) if matches!(u.op, UnaryOp::Affine { .. }))
    // }

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
        let mut e = Expr::lit(3.14f32);
        assert!((f32_val(e.eval(&metrics()).unwrap()) - 3.14).abs() < 1e-6);
    }

    #[test]
    fn lit_ignores_input() {
        let mut e = Expr::lit(42.0f32);
        // Same result regardless of what the input is
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 42.0);
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 42.0);
    }

    // ---- Unary ops ----

    #[test]
    fn neg_negates_numeric() {
        let mut e = Expr::lit(5.0f32).neg();
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), -5.0);
    }

    #[test]
    fn abs_returns_magnitude() {
        let mut e = Expr::lit(-7.0f32).abs();
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 7.0);
    }

    #[test]
    fn not_inverts_bool() {
        let mut t = Expr::lit(AnyValue::Bool(true)).not();
        let mut f = Expr::lit(AnyValue::Bool(false)).not();
        assert!(!bool_val(t.eval(&metrics()).unwrap()));
        assert!(bool_val(f.eval(&metrics()).unwrap()));
    }

    #[test]
    fn not_on_non_bool_errors() {
        let mut e = Expr::lit(1.0f32).not();
        assert!(e.eval(&metrics()).is_err());
    }

    #[test]
    fn cast_f32_to_i32_truncates() {
        let mut e = Expr::lit(3.9f32).cast(DataType::Int32);
        let result = e.eval(&metrics()).unwrap();
        assert_eq!(result.extract::<i32>(), Some(3));
    }

    // ---- Arithmetic binary ops ----

    #[test]
    fn add_two_literals() {
        let mut e = Expr::lit(2.0f32).add(Expr::lit(3.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 5.0);
    }

    #[test]
    fn sub_two_literals() {
        let mut e = Expr::lit(10.0f32).sub(Expr::lit(3.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 7.0);
    }

    #[test]
    fn mul_two_literals() {
        let mut e = Expr::lit(4.0f32) * 2.5f32;
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 10.0);
    }

    #[test]
    fn div_two_literals() {
        let mut e = Expr::lit(9.0f32).div(Expr::lit(3.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 3.0);
    }

    #[test]
    fn pow_two_literals() {
        let mut e = Expr::lit(2.0f32).pow(Expr::lit(8.0f32));
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
        let mut e = !Expr::lit(AnyValue::Bool(true));
        assert!(!bool_val(e.eval(&metrics()).unwrap()));
    }

    // ---- Comparison ops ----

    #[test]
    fn lt_lte_gt_gte_correct() {
        let five = || Expr::lit(5.0f32);
        let ten = || Expr::lit(10.0f32);
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
            Expr::lit(5.0f32).eq(Expr::lit(5.0f32)).eval(input).unwrap()
        ));
        assert!(!bool_val(
            Expr::lit(5.0f32).eq(Expr::lit(6.0f32)).eval(input).unwrap()
        ));
        assert!(bool_val(
            Expr::lit(5.0f32).ne(Expr::lit(6.0f32)).eval(input).unwrap()
        ));
    }

    #[test]
    fn between_is_inclusive_on_both_ends() {
        let input = &metrics();
        let range = || (Expr::lit(1.0f32), Expr::lit(10.0f32));

        let (lo, hi) = range();
        assert!(bool_val(
            Expr::lit(5.0f32).between(lo, hi).eval(input).unwrap()
        ));

        let (lo, hi) = range();
        assert!(bool_val(
            Expr::lit(1.0f32).between(lo, hi).eval(input).unwrap()
        ));

        let (lo, hi) = range();
        assert!(bool_val(
            Expr::lit(10.0f32).between(lo, hi).eval(input).unwrap()
        ));

        let (lo, hi) = range();
        assert!(!bool_val(
            Expr::lit(0.0f32).between(lo, hi).eval(input).unwrap()
        ));
    }

    // ---- Logical ops ----

    #[test]
    fn and_or_short_circuit_values() {
        let input = &metrics();
        let t = || Expr::lit(AnyValue::Bool(true));
        let f = || Expr::lit(AnyValue::Bool(false));

        assert!(!bool_val(t().and(f()).eval(input).unwrap()));
        assert!(bool_val(t().and(t()).eval(input).unwrap()));
        assert!(bool_val(f().or(t()).eval(input).unwrap()));
        assert!(!bool_val(f().or(f()).eval(input).unwrap()));
    }

    // ---- When / then / otherwise ----

    #[test]
    fn when_selects_then_branch_on_true() {
        let mut e = Expr::when(Expr::lit(AnyValue::Bool(true)))
            .then(Expr::lit(1.0f32))
            .otherwise(Expr::lit(2.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 1.0);
    }

    #[test]
    fn when_selects_otherwise_branch_on_false() {
        let mut e = Expr::when(Expr::lit(AnyValue::Bool(false)))
            .then(Expr::lit(1.0f32))
            .otherwise(Expr::lit(2.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 2.0);
    }

    #[test]
    fn when_condition_can_be_a_comparison() {
        let mut e = Expr::when(Expr::lit(5.0f32).gt(Expr::lit(3.0f32)))
            .then(Expr::lit(100.0f32))
            .otherwise(Expr::lit(0.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 100.0);
    }

    // ---- Clamp ----

    #[test]
    fn clamp_below_min_returns_min() {
        let mut e = Expr::lit(-5.0f32).clamp(Expr::lit(0.0f32), Expr::lit(1.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 0.0);
    }

    #[test]
    fn clamp_above_max_returns_max() {
        let mut e = Expr::lit(10.0f32).clamp(Expr::lit(0.0f32), Expr::lit(1.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 1.0);
    }

    #[test]
    fn clamp_within_range_unchanged() {
        let mut e = Expr::lit(0.5f32).clamp(Expr::lit(0.0f32), Expr::lit(1.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 0.5);
    }

    #[test]
    fn clamp_null_input_returns_min() {
        let mut e = Expr::lit(AnyValue::Null).clamp(Expr::lit(0.05f32), Expr::lit(2.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 0.05);
    }

    #[test]
    fn clamp_nan_input_returns_min() {
        let mut e = Expr::lit(f32::NAN).clamp(Expr::lit(0.05f32), Expr::lit(2.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 0.05);
    }

    #[test]
    fn clamp_pos_inf_input_returns_min() {
        let mut e = Expr::lit(f32::INFINITY).clamp(Expr::lit(0.05f32), Expr::lit(2.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 0.05);
    }

    #[test]
    fn clamp_neg_inf_input_returns_min() {
        let mut e = Expr::lit(f32::NEG_INFINITY).clamp(Expr::lit(0.05f32), Expr::lit(2.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 0.05);
    }

    #[test]
    fn clamp_missing_bounds_errors() {
        let mut e = Expr::lit(0.5f32).clamp(Expr::lit(AnyValue::Null), Expr::lit(2.0f32));
        assert!(e.eval(&metrics()).is_err());
    }

    // ---- or_else (Coalesce) ----

    #[test]
    fn or_else_finite_passes_through() {
        let mut e = Expr::lit(3.0f32).or_else(Expr::lit(99.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 3.0);
    }

    #[test]
    fn or_else_null_falls_back() {
        let mut e = Expr::lit(AnyValue::Null).or_else(Expr::lit(99.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 99.0);
    }

    #[test]
    fn or_else_nan_falls_back() {
        let mut e = Expr::lit(f32::NAN).or_else(Expr::lit(99.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 99.0);
    }

    #[test]
    fn or_else_inf_falls_back() {
        let mut e = Expr::lit(f32::INFINITY).or_else(Expr::lit(99.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 99.0);
    }

    #[test]
    fn or_else_neg_inf_falls_back() {
        let mut e = Expr::lit(f32::NEG_INFINITY).or_else(Expr::lit(99.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 99.0);
    }

    #[test]
    fn or_else_chains_through_bad_values() {
        let mut e = Expr::lit(AnyValue::Null)
            .or_else(Expr::lit(f32::NAN))
            .or_else(Expr::lit(7.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 7.0);
    }

    // ---- min_with / max_with ----

    #[test]
    fn min_with_picks_smaller() {
        let mut e = Expr::lit(5.0f32).min_with(Expr::lit(3.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 3.0);
    }

    #[test]
    fn max_with_picks_larger() {
        let mut e = Expr::lit(5.0f32).max_with(Expr::lit(8.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 8.0);
    }

    #[test]
    fn min_with_nan_on_one_side_returns_other() {
        // f32::min(a, NaN) = a (IEEE 754-2019 minNum semantics)
        let mut e = Expr::lit(5.0f32).min_with(Expr::lit(f32::NAN));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 5.0);
    }

    #[test]
    fn max_with_nan_on_one_side_returns_other() {
        let mut e = Expr::lit(5.0f32).max_with(Expr::lit(f32::NAN));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 5.0);
    }

    #[test]
    fn floor_via_max_with_constant() {
        // Common pattern: max_with as a floor without an upper ceiling.
        let mut e = Expr::lit(-3.0f32).max_with(Expr::lit(0.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 0.0);
    }

    // ---- Expr::reset ----

    #[test]
    fn reset_clears_schedule_counter() {
        // every(3) fires true on every third call. After two calls + reset,
        // the next call should NOT fire (counter starts fresh).
        let mut e = Expr::every(3)
            .then(Expr::lit(1.0f32))
            .otherwise(Expr::lit(0.0f32));

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
        let mut e = Expr::lit(42.0f32);
        e.reset();
        e.reset();
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 42.0);
    }

    // ---- Schedule: every(n) ----

    #[test]
    fn every_fires_on_nth_call_then_resets() {
        let mut e = Expr::every(3)
            .then(Expr::lit(AnyValue::Bool(true)))
            .otherwise(Expr::lit(AnyValue::Bool(false)));

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
        let e = Expr::lit(15.0f32).error(10.0);
        // assert!(is_fused_affine(&e), "expected fused Affine, got {e:?}");
        let mut e = e;
        assert!((f32_val(e.eval(&metrics()).unwrap()) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn error_from_function_reads_metric() {
        let ms = metrics_with("foo", 12.0);
        let mut e = Expr::select("foo").error(10.0);
        // (12 - 10) / 10 = 0.2
        assert!((f32_val(e.eval(&ms).unwrap()) - 0.2).abs() < 1e-6);
    }

    // ---- Streaming quantile (P²) ----

    #[test]
    fn quantile_stream_returns_first_sample_until_buffer_fills() {
        let mut e = Expr::select("foo").quantile(0.5);
        let ms = metrics_with("foo", 5.0);
        // First sample seeds the estimator; with one sample p50 == that sample.
        assert!((f32_val(e.eval(&ms).unwrap()) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn quantile_stream_null_when_metric_missing() {
        let mut e = Expr::select("missing").quantile(0.95);
        let ms = MetricSet::new();
        assert!(matches!(e.eval(&ms).unwrap(), AnyValue::Null));
    }

    #[test]
    fn quantile_stream_converges_on_uniform_sequence() {
        let mut e = Expr::select("foo").quantile(0.5);
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
        let mut e = Expr::select("foo").quantile(0.95);
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
        let mut e = Expr::select("foo").quantile(0.5);
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
        let mut e = Expr::lit(42.0f32).quantile(0.5);
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
        let mut e = Expr::select("score").stagnation(0.001);

        assert_eq!(f32_val(e.eval(&ms).unwrap()), 0.0); // seed
        assert_eq!(f32_val(e.eval(&ms).unwrap()), 1.0);
        assert_eq!(f32_val(e.eval(&ms).unwrap()), 2.0);
    }

    #[test]
    fn stagnation_resets_on_large_change() {
        let mut ms = metrics_with("score", 1.0);
        let mut e = Expr::select("score").stagnation(0.001);

        let _ = e.eval(&ms);
        let _ = e.eval(&ms); // count = 1

        ms.upsert("score", 5.0); // big change > epsilon
        assert_eq!(f32_val(e.eval(&ms).unwrap()), 0.0);
        assert_eq!(f32_val(e.eval(&ms).unwrap()), 1.0);
    }

    #[test]
    fn stagnation_tolerates_tiny_noise() {
        let mut ms = metrics_with("score", 1.0);
        let mut e = Expr::select("score").stagnation(0.01);

        let _ = e.eval(&ms);
        ms.upsert("score", 1.005); // within epsilon
        assert_eq!(f32_val(e.eval(&ms).unwrap()), 1.0);
        ms.upsert("score", 1.008);
        assert_eq!(f32_val(e.eval(&ms).unwrap()), 2.0);
    }

    #[test]
    fn stagnation_returns_null_when_metric_missing() {
        let ms = MetricSet::new();
        let mut e = Expr::select("missing").stagnation(0.001);
        assert!(matches!(e.eval(&ms).unwrap(), AnyValue::Null));
    }

    #[test]
    fn is_stagnant_fires_at_patience_threshold() {
        let ms = metrics_with("score", 1.0);
        let mut e = Expr::select("score").stagnation(0.001).gte(3);

        assert!(!bool_val(e.eval(&ms).unwrap())); // count=0
        assert!(!bool_val(e.eval(&ms).unwrap())); // count=1
        assert!(!bool_val(e.eval(&ms).unwrap())); // count=2
        assert!(bool_val(e.eval(&ms).unwrap())); // count=3, fires
    }

    #[test]
    fn stagnation_reset_clears_state() {
        let ms = metrics_with("score", 1.0);
        let mut e = Expr::select("score").stagnation(0.001);

        let _ = e.eval(&ms);
        let _ = e.eval(&ms);
        let _ = e.eval(&ms); // count = 2

        e.reset();
        assert_eq!(f32_val(e.eval(&ms).unwrap()), 0.0); // fresh seed
    }

    #[test]
    fn is_converged_fires_when_window_is_flat() {
        // let ms = metrics_with("score", 1.0);
        // let mut e = Expr::is_converged("score", 3, 0.01);

        // // Buffers seed up to size 3.
        // let _ = e.eval(&ms);
        // let _ = e.eval(&ms);
        // // Third eval: both first and last buffers full, both should hold ~1.0.
        // assert!(bool_val(e.eval(&ms).unwrap()));
    }

    #[test]
    fn is_converged_does_not_fire_when_window_drifts() {
        // let mut ms = metrics_with("score", 1.0);
        // let mut e = Expr::is_converged("score", 3, 0.01);

        // let _ = e.eval(&ms);
        // ms.upsert("score", 2.0);
        // let _ = e.eval(&ms);
        // ms.upsert("score", 3.0);
        // // first=1.0, last=3.0, diff=2.0 > epsilon
        // assert!(!bool_val(e.eval(&ms).unwrap()));
    }

    // ---- compile() ----

    #[test]
    fn compile_folds_pure_literal_subtree() {
        let e = Expr::lit(2.0f32).add(Expr::lit(3.0f32)).compile();
        assert!(matches!(e.kind(), ExprKind::Literal(_)));
        let mut e = e;
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 5.0);
    }

    // #[test]
    // fn compile_wraps_metric_plus_lit_as_affine() {
    //     let e = Expr::select("foo").add(Expr::lit(3.0f32)).compile();
    //     assert!(is_fused_affine(&e), "expected Affine, got {e:?}");
    // }

    // #[test]
    // fn compile_collapses_controller_chain_to_single_affine() {
    //     // Replicates the species-threshold count_error chain:
    //     //   (x - target) / target * GAIN + 1.0
    //     //   = x * (GAIN / target) + (1 - GAIN)
    //     let target = 10.0f32;
    //     let gain = 0.999f32;
    //     let e = Expr::select("count.species")
    //         .sub(Expr::lit(target))
    //         .div(Expr::lit(target))
    //         .mul(Expr::lit(gain))
    //         .add(Expr::lit(1.0f32))
    //         .compile();

    //     assert!(is_fused_affine(&e), "expected single Affine, got {e:?}");
    // }

    #[test]
    fn compile_is_idempotent() {
        let e = Expr::select("foo")
            .sub(Expr::lit(1.0f32))
            .mul(Expr::lit(2.0f32))
            .add(Expr::lit(3.0f32));
        let once = e.clone().compile();
        let twice = once.clone().compile();
        assert_eq!(format!("{:?}", once), format!("{:?}", twice));
    }

    // ---- Affine ----

    // ---- Composition ----

    #[test]
    fn composed_expr_add_then_compare() {
        // (2 + 3) > 4 → true
        let mut e = Expr::lit(2.0f32)
            .add(Expr::lit(3.0f32))
            .gt(Expr::lit(4.0f32));
        assert!(bool_val(e.eval(&metrics()).unwrap()));
    }

    #[test]
    fn composed_expr_clamp_then_scale() {
        // clamp(-5, 0, 1) * 10 → 0.0
        let mut e = Expr::lit(-5.0f32)
            .clamp(Expr::lit(0.0f32), Expr::lit(1.0f32))
            .mul(Expr::lit(10.0f32));
        assert_eq!(f32_val(e.eval(&metrics()).unwrap()), 0.0);
    }

    #[test]
    fn test_identity_select() {
        let mut e = Expr::identity().rolling(3).sum();

        for i in 0..5 {
            let output = e.eval(&i);

            if i < 2 {
                assert_eq!(f32_val(output.unwrap()), (0..=i).sum::<i32>() as f32);
            } else {
                assert_eq!(f32_val(output.unwrap()), (i - 2..=i).sum::<i32>() as f32);
            }
        }
    }
}
