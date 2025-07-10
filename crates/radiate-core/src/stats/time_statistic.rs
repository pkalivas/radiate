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
