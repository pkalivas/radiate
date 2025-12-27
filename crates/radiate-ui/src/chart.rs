use radiate_utils::WindowBuffer;
use ratatui::style::Color;

pub struct RollingChart {
    title: String,
    min_y: f64,
    max_y: f64,
    values: WindowBuffer<(f64, f64)>,
    color: Color,
    point_count: usize,
}

impl RollingChart {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            title: "".to_string(),
            min_y: f64::MAX,
            max_y: f64::MIN,
            values: WindowBuffer::with_window(capacity),
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

        if self.min_y == self.max_y {
            // avoid zero range
            self.min_y -= 0.5;
            self.max_y += 0.5;
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
}
