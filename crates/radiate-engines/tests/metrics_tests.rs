#[cfg(test)]
mod tests {
    use radiate_core::*;
    use radiate_engines::*;

    const ROUND_PLACES: i32 = 5;

    fn round_value(value: f32) -> f32 {
        let factor = 10_f32.powi(ROUND_PLACES);
        (value * factor).round() / factor
    }

    #[test]
    fn metrics_recorded_during_engine_run() {
        random_provider::set_seed(12345);

        const MIN_SCORE: f32 = 0.00;
        const A: f32 = 10.0;
        const RANGE: f32 = 5.12;
        const N_GENES: usize = 2;

        let engine = GeneticEngine::builder()
            .codec(FloatChromosome::from((N_GENES, -RANGE..RANGE)))
            .minimizing()
            .population_size(500)
            .alter(alters!(
                UniformCrossover::new(0.5),
                ArithmeticMutator::new(0.01)
            ))
            .fitness_fn(move |genotype: Vec<f32>| {
                let mut value = A * N_GENES as f32;
                for i in 0..N_GENES {
                    value +=
                        genotype[i].powi(2) - A * (2.0 * std::f32::consts::PI * genotype[i]).cos();
                }

                value
            })
            .build();

        let result = engine
            .iter()
            .until_score(MIN_SCORE)
            .last()
            .expect("Engine did not produce a result");

        let engine_metrics = result.metrics();

        let metrics_path = std::env::current_dir()
            .expect("Failed to get current directory")
            .join("tests/data/engine_metrics.json");

        let loaded_metrics = serde_json::from_str::<MetricSet>(
            &std::fs::read_to_string(&metrics_path)
                .expect("Failed to read engine metrics from file"),
        )
        .expect("Failed to deserialize engine metrics");

        for key in engine_metrics.keys() {
            let engine_metric = engine_metrics.get(key).expect("Engine metric missing key");
            let loaded_metric = loaded_metrics.get(key).expect("Loaded metric missing key");

            assert!(
                engine_metric.name() == loaded_metric.name(),
                "Metric names do not match for key: {}",
                key
            );

            assert_statistics(engine_metric, loaded_metric, key);
            assert_time_statistics(engine_metric, loaded_metric, key);
        }
    }

    fn assert_time_statistics(engine_metric: &Metric, loaded_metric: &Metric, key: &str) {
        let engine_statistics = engine_metric.time_statistic();
        let loaded_statistics = loaded_metric.time_statistic();

        if let Some(engine_statistics) = engine_statistics {
            if let Some(loaded_statistics) = loaded_statistics {
                // Only compare count for time metrics as others may vary due to execution time differences
                assert!(
                    engine_statistics.count() == loaded_statistics.count(),
                    "Count values do not match for time metric: {}",
                    key
                );
            } else {
                panic!("Loaded time metric has no statistics");
            }
        }
    }

    fn assert_statistics(engine_metric: &Metric, loaded_metric: &Metric, key: &str) {
        let engine_statistics = engine_metric.statistic();
        let loaded_statistics = loaded_metric.statistic();

        if let Some(engine_statistics) = engine_statistics {
            if let Some(loaded_statistics) = loaded_statistics {
                // Statistic comparisons
                assert!(
                    round_value(engine_statistics.min()) == round_value(loaded_statistics.min()),
                    "Min values do not match for metric: {}",
                    key
                );

                assert!(
                    round_value(engine_statistics.max()) == round_value(loaded_statistics.max()),
                    "Max values do not match for metric: {}",
                    key
                );

                assert!(
                    round_value(engine_statistics.mean()) == round_value(loaded_statistics.mean()),
                    "Mean values do not match for metric: {}",
                    key
                );

                assert!(
                    round_value(engine_statistics.std_dev())
                        == round_value(loaded_statistics.std_dev()),
                    "Std Dev values do not match for metric: {}",
                    key
                );

                assert!(
                    round_value(engine_statistics.sum()) == round_value(loaded_statistics.sum()),
                    "Sum values do not match for metric: {}",
                    key
                );

                assert!(
                    engine_statistics.count() == loaded_statistics.count(),
                    "Count values do not match for metric: {}",
                    key
                );

                assert!(
                    round_value(engine_statistics.variance())
                        == round_value(loaded_statistics.variance()),
                    "Variance values do not match for metric: {}",
                    key
                );

                assert!(
                    round_value(engine_statistics.kurtosis())
                        == round_value(loaded_statistics.kurtosis()),
                    "Kurtosis values do not match for metric: {}",
                    key
                );

                assert!(
                    round_value(engine_statistics.skewness())
                        == round_value(loaded_statistics.skewness()),
                    "Skewness values do not match for metric: {}",
                    key
                );

                assert!(
                    round_value(engine_statistics.last_value())
                        == round_value(loaded_statistics.last_value()),
                    "Last Value values do not match for metric: {}",
                    key
                );
            } else {
                panic!("Loaded metric has no statistics for key: {}", key);
            }
        }
    }
}
