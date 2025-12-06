use ratatui::style::Color;

pub struct ChartData {
    pub name: String,
    pub min_y: f64,
    pub max_y: f64,
    pub values: Vec<(f64, f64)>,
    pub color: Color,
}

impl ChartData {
    pub fn new() -> Self {
        Self {
            name: "".to_string(),
            min_y: f64::MAX,
            max_y: f64::MIN,
            values: Vec::new(),
            color: Color::White,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = name.into();
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn min_y(&self) -> f64 {
        self.min_y
    }

    pub fn max_y(&self) -> f64 {
        self.max_y
    }

    pub fn min_x(&self) -> f64 {
        if let Some((x, _)) = self.values.first() {
            *x
        } else {
            0.0
        }
    }

    pub fn max_x(&self) -> f64 {
        if let Some((x, _)) = self.values.last() {
            *x
        } else {
            0.0
        }
    }

    pub fn values(&self) -> &Vec<(f64, f64)> {
        &self.values
    }

    pub fn add_value(&mut self, value: (f64, f64)) {
        if value.1 < self.min_y {
            self.min_y = value.1;
        }
        if value.1 > self.max_y {
            self.max_y = value.1;
        }
        self.values.push(value);
    }

    #[allow(dead_code)]
    pub fn set_values(&mut self, values: &[f32]) {
        self.min_y = f64::MAX;
        self.max_y = f64::MIN;
        self.values.clear();

        for (i, val) in values.iter().enumerate() {
            let f_val = *val as f64;
            if f_val < self.min_y {
                self.min_y = f_val;
            }
            if f_val > self.max_y {
                self.max_y = f_val;
            }

            self.values.push((i as f64, f_val));
        }
    }
}

impl Default for ChartData {
    fn default() -> Self {
        Self {
            name: "Score".to_string(),
            min_y: f64::MAX,
            max_y: f64::MIN,
            values: Vec::new(),
            color: Color::White,
        }
    }
}
