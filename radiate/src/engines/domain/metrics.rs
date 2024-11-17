use super::Statistic;

pub struct Metric {
    pub name: &'static str,
    pub stats: Statistic,
}

impl Metric {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            stats: Statistic::new(),
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn add(&mut self, value: f32) {
        self.stats.add(value);
    }

    pub fn last_value(&self) -> f32 {
        self.stats.last_value()
    }

    pub fn variance(&self) -> f32 {
        self.stats.variance()
    }

    pub fn skewness(&self) -> f32 {
        self.stats.skewness()
    }

    pub fn std_dev(&self) -> f32 {
        self.stats.std_dev()
    }

    pub fn mean(&self) -> f32 {
        self.stats.mean()
    }

    pub fn min(&self) -> f32 {
        self.stats.min()
    }

    pub fn max(&self) -> f32 {
        self.stats.max()
    }

    pub fn count(&self) -> i32 {
        self.stats.count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric() {
        let mut metric = Metric::new("test");
        metric.add(1.0);
        metric.add(2.0);
        metric.add(3.0);
        metric.add(4.0);
        metric.add(5.0);

        assert_eq!(metric.count(), 5);
        assert_eq!(metric.last_value(), 5.0);
        assert_eq!(metric.mean(), 3.0);
        assert_eq!(metric.variance(), 2.5);
        assert_eq!(metric.std_dev(), 1.5811388);
        assert_eq!(metric.skewness(), 0.0);
        assert_eq!(metric.min(), 1.0);
        assert_eq!(metric.max(), 5.0);
        assert_eq!(metric.name(), "test");
    }
}
