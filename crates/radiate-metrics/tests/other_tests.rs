#[cfg(test)]
mod tests {
    use std::time::Duration;

    use radiate_metrics::{ExprSelect, Metric, MetricUpdate, Selector, TagType, metric_fields};
    use radiate_utils::{AnyValue, SmallStr};

    // ... existing tests ...

    fn populated(name: &str) -> Metric {
        let mut m = Metric::new(name);
        for v in [1.0, 2.0, 3.0, 4.0, 5.0] {
            m.apply_update(v);
        }
        m
    }

    #[test]
    fn select_last_value_field() {
        let m = populated("x");
        assert_eq!(
            m.select(&Selector::Field(metric_fields::LAST_VALUE)),
            AnyValue::Float32(5.0)
        );
    }

    #[test]
    fn select_mean_and_stddev_fields() {
        let m = populated("x");
        assert_eq!(
            m.select(&Selector::Field(metric_fields::MEAN)),
            AnyValue::Float32(3.0)
        );
        assert_eq!(
            m.select(&Selector::Field(metric_fields::STDDEV)),
            AnyValue::Float32(1.5811388)
        );
    }

    #[test]
    fn select_count_generation_update_count_are_always_uint64() {
        let m = populated("x");
        assert_eq!(
            m.select(&Selector::Field(metric_fields::COUNT)),
            AnyValue::UInt64(5)
        );
        assert_eq!(
            m.select(&Selector::Field(metric_fields::GENERATION)),
            AnyValue::UInt64(0)
        );
        assert_eq!(
            m.select(&Selector::Field(metric_fields::UPDATE_COUNT)),
            AnyValue::UInt64(5)
        );
    }

    #[test]
    fn select_skew_and_kurt_route_through_dtype_wrap() {
        // On a Float32-dtype metric this stays Float32 — but it's now going
        // through the same `wrap()` path as mean/stddev/etc, not a bypass.
        let m = populated("x");
        assert_eq!(
            m.select(&Selector::Field(metric_fields::SKEWNESS)),
            AnyValue::Float32(m.skew())
        );
        assert_eq!(
            m.select(&Selector::Field(metric_fields::KURTOSIS)),
            AnyValue::Float32(m.kurt())
        );
    }

    #[test]
    fn select_unknown_field_returns_null() {
        let m = populated("x");
        assert_eq!(
            m.select(&Selector::Field(SmallStr::from_static("not_a_field"))),
            AnyValue::Null
        );
    }

    #[test]
    fn select_non_field_selector_returns_null() {
        // Metric only understands Selector::Field — Identity/Index/Range/Nested
        // are the outer MetricSet's job to resolve before reaching here.
        let m = populated("x");
        assert_eq!(m.select(&Selector::Identity), AnyValue::Null);
        assert_eq!(m.select(&Selector::Index(0)), AnyValue::Null);
    }

    #[test]
    fn generation_bump_resets_update_count_but_not_stats() {
        let mut m = populated("x");
        assert_eq!(m.update_count(), 5);

        m.set_generation(1);
        assert_eq!(m.update_count(), 0);
        assert_eq!(
            m.count(),
            5,
            "underlying Statistic should survive a generation bump"
        );

        m.apply_update(6.0);
        assert_eq!(m.update_count(), 1);
    }

    #[test]
    fn clear_values_resets_stats_and_samples() {
        let mut m = Metric::new("x");
        m.apply_update(&[1.0, 2.0, 3.0][..]);
        assert!(m.count() > 0);

        m.clear_values();
        assert_eq!(m.count(), 0);
        assert!(m.stats().is_none());
    }

    #[test]
    fn distributions_view_none_without_distribution_tag() {
        let m = populated("x"); // scalar updates only -> Statistic tag, not Distribution
        assert!(m.distributions().is_none());
    }

    #[test]
    fn distributions_view_some_with_quantile_after_slice_update() {
        let mut m = Metric::new("x");
        m.apply_update(&[1.0, 2.0, 3.0, 4.0, 5.0][..]);
        assert!(m.distributions().is_some());
        assert_eq!(m.quantile(0.5), Some(3.0));
    }

    #[test]
    fn quantile_none_without_distribution_tag() {
        let m = populated("x");
        assert_eq!(m.quantile(0.5), None);
    }

    #[test]
    fn bool_update_maps_to_zero_or_one() {
        let mut m = Metric::new("flag");
        m.apply_update(true);
        m.apply_update(false);
        assert_eq!(m.mean(), 0.5);
    }

    #[test]
    fn metric_update_from_conversions_roundtrip() {
        assert_eq!(MetricUpdate::from(1.5_f32), MetricUpdate::Float(1.5));
        assert_eq!(MetricUpdate::from(3_usize), MetricUpdate::Usize(3));
        assert_eq!(MetricUpdate::from(true), MetricUpdate::Bool(true));
        assert_eq!(
            MetricUpdate::from(Duration::from_secs(1)),
            MetricUpdate::Duration(Duration::from_secs(1))
        );
    }

    #[test]
    fn try_from_anyvalue_numeric_variants_normalize_to_float() {
        assert_eq!(
            MetricUpdate::try_from(AnyValue::UInt32(7)).unwrap(),
            MetricUpdate::Float(7.0)
        );
        assert_eq!(
            MetricUpdate::try_from(AnyValue::Int64(-3)).unwrap(),
            MetricUpdate::Float(-3.0)
        );
        assert_eq!(
            MetricUpdate::try_from(AnyValue::Float64(2.5)).unwrap(),
            MetricUpdate::Float(2.5)
        );
    }

    #[test]
    fn try_from_anyvalue_duration_and_bool_pass_through() {
        assert_eq!(
            MetricUpdate::try_from(AnyValue::Duration(Duration::from_secs(4))).unwrap(),
            MetricUpdate::Duration(Duration::from_secs(4))
        );
        assert_eq!(
            MetricUpdate::try_from(AnyValue::Bool(true)).unwrap(),
            MetricUpdate::Bool(true)
        );
    }

    #[test]
    fn try_from_anyvalue_vector_of_numerics_becomes_owned_distribution() {
        let values = vec![
            AnyValue::Float32(1.0),
            AnyValue::Int32(2),
            AnyValue::UInt8(3),
        ];
        let update = MetricUpdate::try_from(AnyValue::Vector(values)).unwrap();
        assert_eq!(update, MetricUpdate::OwnedDistribution(vec![1.0, 2.0, 3.0]));
    }

    #[test]
    fn try_from_anyvalue_vector_with_non_numeric_element_errors() {
        let values = vec![AnyValue::Float32(1.0), AnyValue::Str("nope".into())];
        assert!(MetricUpdate::try_from(AnyValue::Vector(values)).is_err());
    }

    #[test]
    fn try_from_anyvalue_unsupported_type_errors() {
        assert!(MetricUpdate::try_from(AnyValue::Str("x".into())).is_err());
    }

    #[test]
    fn update_from_uses_sum_shortcut_for_plain_statistics() {
        // count == sum and no Distribution tag -> takes the `apply_update(sum)`
        // fast path documented on update_from, not the full Statistic merge.
        let mut base = Metric::new("x");
        base.apply_update(2.0);

        let mut incoming = Metric::new("x");
        incoming.apply_update(1.0); // count=1, sum=1.0 -> count as f32 == sum

        base.update_from(incoming);
        assert_eq!(base.sum(), 3.0); // 1 (base) + 1 (fast-path add) + ... see note below
    }

    #[test]
    fn update_from_merges_full_statistic_for_distributions() {
        let mut base = Metric::new("x");
        base.apply_update(&[1.0, 2.0][..]);

        let mut incoming = Metric::new("x");
        incoming.apply_update(&[3.0, 4.0, 5.0][..]);

        base.update_from(incoming);
        assert_eq!(base.count(), 5);
        assert_eq!(base.sum(), 15.0);
        assert!(base.tags().has(TagType::Distribution));
    }
}
