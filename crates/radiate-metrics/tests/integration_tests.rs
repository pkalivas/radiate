#[cfg(test)]
mod expr_integration_tests {

    use radiate_metrics::{EvalExpr, Expr, MetricSet};
    use radiate_utils::AnyValue;

    #[test]
    fn expr_metric_reads_last_value_by_default() {
        let mut set = MetricSet::new();
        set.upsert("species.count", 3.0_f32);
        set.upsert("species.count", 8.0_f32);

        let mut expr = Expr::metric("species.count");
        assert_eq!(expr.evaluate(&set).unwrap(), AnyValue::Float32(8.0));
    }

    #[test]
    fn expr_metric_field_swap_reads_mean_instead() {
        let mut set = MetricSet::new();
        for v in [1.0, 2.0, 3.0] {
            set.upsert("species.count", v);
        }

        let mut expr = Expr::metric("species.count").mean();
        assert_eq!(expr.evaluate(&set).unwrap(), AnyValue::Float32(2.0));
    }

    #[test]
    fn expr_rolling_aggregate_over_ticks() {
        let mut set = MetricSet::new();
        set.upsert("score", 10.0_f32);

        let mut expr = Expr::metric("score").rolling(3).mean();
        // Each .evaluate() call re-reads "score" from `set` and pushes into
        // the rolling buffer — set doesn't change here, so this exercises
        // the buffer filling up with the same repeated value.
        let mut last = AnyValue::Null;
        for _ in 0..3 {
            last = expr.evaluate(&set).unwrap();
        }
        assert_eq!(last, AnyValue::Float32(10.0));
    }

    #[test]
    fn expr_unknown_metric_name_is_null_and_coalesce_falls_back() {
        let set = MetricSet::new();
        let mut expr = Expr::metric("does.not.exist").coalesce(Expr::lit(42.0_f32));
        assert_eq!(expr.evaluate(&set).unwrap(), AnyValue::Float32(42.0));
    }

    #[test]
    fn expr_error_signal_against_live_metric() {
        let mut set = MetricSet::new();
        set.upsert("species.count", 12.0_f32);

        let mut expr = Expr::metric("species.count").error(10.0);
        // (12 - 10) / 10 == 0.2
        let AnyValue::Float32(v) = expr.evaluate(&set).unwrap() else {
            panic!("expected Float32");
        };
        assert!((v - 0.2).abs() < 1e-6);
    }
}
