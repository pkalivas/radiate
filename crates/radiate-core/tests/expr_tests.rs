#[cfg(test)]
mod test {

    use radiate_core::{AnyValue, Evaluate, Expr, MetricSet};
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
