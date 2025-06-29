use crate::Statistic;
use std::time::Duration;

#[derive(Clone, PartialEq, Default)]
pub struct TimeStatistic {
    pub statistic: Statistic,
    pub last_time: Duration,
}

impl TimeStatistic {
    pub fn new(initial_val: Duration) -> Self {
        let mut result = TimeStatistic::default();
        result.add(initial_val);
        result
    }

    pub fn add(&mut self, value: Duration) {
        self.statistic.add(value.as_secs_f32());
        self.last_time = value;
    }

    pub fn last_time(&self) -> Duration {
        self.last_time
    }

    pub fn count(&self) -> i32 {
        self.statistic.count()
    }

    pub fn mean(&self) -> Duration {
        Duration::from_secs_f32(self.statistic.mean())
    }

    pub fn variance(&self) -> Duration {
        Duration::from_secs_f32(self.statistic.variance())
    }

    pub fn standard_deviation(&self) -> Duration {
        Duration::from_secs_f32(self.statistic.std_dev())
    }

    pub fn min(&self) -> Duration {
        Duration::from_secs_f32(self.statistic.min())
    }

    pub fn max(&self) -> Duration {
        Duration::from_secs_f32(self.statistic.max())
    }

    pub fn sum(&self) -> Duration {
        Duration::from_secs_f32(self.statistic.sum())
    }

    pub fn clear(&mut self) {
        self.statistic.clear();
    }
}

impl From<Duration> for TimeStatistic {
    fn from(value: Duration) -> Self {
        TimeStatistic::new(value)
    }
}

// // Add to crates/radiate-core/src/stats/derived.rs
// pub fn register_builtin_derived(registry: &mut DerivedMetricRegistry) {
//     // Evaluations per second
//     registry.register("derived_evals_per_sec", |metrics| {
//         let eval_count = metrics.get("Fitness")?.count() as f32;
//         let eval_time = metrics.get("Fitness")?.time_sum()?.as_secs_f32();
//         if eval_time > 0.0 {
//             Some(Metric::new_value("derived_evals_per_sec").with_value(eval_count / eval_time))
//         } else {
//             None
//         }
//     });

//     // Fitness improvement rate
//     registry.register("derived_fitness_improvement", |metrics| {
//         let current_fitness = metrics.get("Score")?.last_value();
//         let prev_fitness = metrics.get("Score")?.value_mean()?;
//         Some(Metric::new_value("derived_fitness_improvement").with_value(current_fitness - prev_fitness))
//     });

//     // Diversity change
//     registry.register("derived_diversity_change", |metrics| {
//         let current_diversity = metrics.get("Unique(members)")?.last_value();
//         let prev_diversity = metrics.get("Unique(members)")?.value_mean()?;
//         Some(Metric::new_value("derived_diversity_change").with_value((current_diversity - prev_diversity).abs()))
//     });
// }
