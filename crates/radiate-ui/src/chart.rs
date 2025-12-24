use ratatui::style::Color;

pub struct RollingBuffer<T> {
    buffer: Vec<T>,
    cap: usize,
    max: usize,
    start: usize,
}

impl<T> RollingBuffer<T> {
    pub fn with_capacity(cap: usize) -> Self {
        assert!(cap > 0, "RollingBuffer capacity must be > 0");

        let max = cap * 2;
        Self {
            buffer: Vec::with_capacity(max),
            cap,
            max,
            start: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, item: T) -> bool {
        let mut resized = false;
        if self.buffer.len() + 1 > self.max {
            self.buffer.drain(0..self.cap);
            self.start = self.buffer.len().saturating_sub(self.cap);
            resized = true;
        }

        self.buffer.push(item);

        let len = self.buffer.len();
        if len > self.cap {
            self.start = len - self.cap;
        } else {
            self.start = 0;
        }

        resized
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len().saturating_sub(self.start)
    }

    #[inline]
    pub fn values(&self) -> &[T] {
        &self.buffer[self.start..]
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.values().iter()
    }
}

pub struct RollingChart {
    title: String,
    min_y: f64,
    max_y: f64,
    values: RollingBuffer<(f64, f64)>,
    color: Color,
    point_count: usize,
}

impl RollingChart {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            title: "".to_string(),
            min_y: f64::MAX,
            max_y: f64::MIN,
            values: RollingBuffer::with_capacity(capacity),
            color: Color::White,
            point_count: 0,
        }
    }

    pub fn with_title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn values(&self) -> &[(f64, f64)] {
        &self.values.values()
    }

    pub fn min_x(&self) -> f64 {
        self.values().first().map_or(0.0, |v| v.0)
    }

    pub fn max_x(&self) -> f64 {
        self.values().last().map_or(0.0, |v| v.0)
    }

    pub fn min_y(&self) -> f64 {
        self.min_y
    }

    pub fn max_y(&self) -> f64 {
        self.max_y
    }

    pub fn push(&mut self, value: f64) {
        let x = self.point_count as f64;
        self.point_count += 1;
        self.add_value((x, value));
    }

    pub fn add_value(&mut self, value: (f64, f64)) {
        let resized = self.values.push(value);

        if resized {
            self.recompute_bounds();
        } else {
            let y = value.1;
            if y < self.min_y {
                self.min_y = y;
            }
            if y > self.max_y {
                self.max_y = y;
            }
        }
    }

    fn recompute_bounds(&mut self) {
        self.min_y = f64::MAX;
        self.max_y = f64::MIN;

        for (_, y) in self.values.iter() {
            if *y < self.min_y {
                self.min_y = *y;
            }
            if *y > self.max_y {
                self.max_y = *y;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::chart::RollingChart;

    #[test]
    fn it_works() {
        let mut chart = RollingChart::with_capacity(5);
        for i in 0..20 {
            chart.add_value((i as f64, i as f64 * i as f64));
            println!(
                "Added value {}, chart len: {:?}",
                i * i,
                (chart.min_y(), chart.max_y())
            );
        }
    }

    #[test]
    fn ring_buffer_works() {
        let mut buffer = super::RollingBuffer::with_capacity(5);
        for i in 0..20 {
            buffer.push(i);
            println!(
                "Added value {}, buffer len: {:?} -> {:?}",
                i,
                buffer.len(),
                buffer.values()
            );
        }
    }
}
