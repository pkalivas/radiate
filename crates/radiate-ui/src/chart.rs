use ratatui::style::Color;

pub struct RollingChart {
    title: String,
    min_y: f64,
    max_y: f64,
    values: Vec<(f64, f64)>,
    color: Color,
    capacity: usize,
}

impl RollingChart {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            title: "".to_string(),
            min_y: f64::MAX,
            max_y: f64::MIN,
            values: Vec::with_capacity(capacity),
            color: Color::White,
            capacity,
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
        &self.values
    }

    pub fn min_x(&self) -> f64 {
        self.values.first().map(|(x, _)| *x).unwrap_or(0.0)
    }

    pub fn max_x(&self) -> f64 {
        self.values.last().map(|(x, _)| *x).unwrap_or(0.0)
    }

    pub fn min_y(&self) -> f64 {
        self.min_y
    }

    pub fn max_y(&self) -> f64 {
        self.max_y
    }

    pub fn push(&mut self, value: f64) {
        let x = self.values.len() as f64;
        self.add_value((x, value));
    }

    pub fn add_value(&mut self, value: (f64, f64)) {
        self.values.push(value);

        if self.values.len() > self.capacity {
            let mut overflow = self.values.len() - self.capacity;
            while overflow > 0 {
                self.values.remove(0);
                overflow -= 1;
            }
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

    // pub fn set_values(&mut self, values: &[f32]) {
    //     self.values.clear();

    //     let keep = values.len().min(self.capacity);
    //     let start = values.len().saturating_sub(keep);
    //     self.min_y = f64::MAX;
    //     self.max_y = f64::MIN;

    //     for (i, val) in values.iter().enumerate().skip(start) {
    //         let f_val = *val as f64;

    //         if f_val < self.min_y {
    //             self.min_y = f_val;
    //         }
    //         if f_val > self.max_y {
    //             self.max_y = f_val;
    //         }

    //         self.values.push((i as f64, f_val));
    //     }
    // }

    fn recompute_bounds(&mut self) {
        self.min_y = f64::MAX;
        self.max_y = f64::MIN;

        for &(_, y) in &self.values {
            if y < self.min_y {
                self.min_y = y;
            }
            if y > self.max_y {
                self.max_y = y;
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
