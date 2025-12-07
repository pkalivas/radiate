use radiate_engines::Metric;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    symbols,
    text::Line,
    widgets::{Axis, Block, Chart, Dataset, GraphType, Widget},
};

pub struct ChartInner {
    title: String,
    min_y: f64,
    max_y: f64,
    values: Vec<(f64, f64)>,
    color: Color,
    capacity: usize,
}

impl ChartInner {
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

    pub fn set_values(&mut self, values: &[f32]) {
        self.values.clear();

        let keep = values.len().min(self.capacity);
        let start = values.len().saturating_sub(keep);
        self.min_y = f64::MAX;
        self.max_y = f64::MIN;

        for (i, val) in values.iter().enumerate().skip(start) {
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

impl Widget for &ChartInner {
    fn render(self, area: Rect, buf: &mut Buffer) {
        chart_widget(
            (self.min_x(), self.max_x()),
            (self.min_y(), self.max_y()),
            vec![&self],
        )
        .render(area, buf)
    }
}

pub struct ChartData {
    name: String,
    metric_name: &'static str,
    last_value: ChartInner,
    mean_value: Option<ChartInner>,
}

impl ChartData {
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            name: "".to_string(),
            metric_name: "",
            last_value: ChartInner::with_capacity(capacity).with_color(Color::Cyan),
            mean_value: None,
        }
    }

    pub fn with_metric_name(mut self, metric_name: &'static str) -> Self {
        self.metric_name = metric_name;
        self.last_value.title = metric_name.into();
        self
    }

    pub fn with_name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = name.into();
        self.last_value.title = self.name.clone();
        self
    }

    pub fn update_last_value(&mut self, index: f64, value: f64) {
        self.last_value.add_value((index, value));
    }

    pub fn update_mean_value(&mut self, index: f64, value: f64) {
        if let Some(mean_chart) = &mut self.mean_value {
            mean_chart.add_value((index, value));
        } else {
            let mut new_inner =
                ChartInner::with_capacity(self.last_value.capacity).with_title("μ (mean)");
            new_inner.add_value((index, value));
            self.mean_value = Some(new_inner);
        }
    }

    pub fn update(&mut self, metric: &Metric) {
        if let Some(stat) = metric.statistic() {
            let len = self.last_value.values.len() as f64;
            self.last_value.add_value((len, stat.last_value() as f64));

            if let Some(mean_chart) = &mut self.mean_value {
                let mean_len = mean_chart.values.len() as f64;
                mean_chart.add_value((mean_len, stat.mean() as f64));
            } else {
                let mut new_inner =
                    ChartInner::with_capacity(self.last_value.capacity).with_title("μ (mean)");
                new_inner.add_value((0.0, stat.mean() as f64));
                self.mean_value = Some(new_inner);
            }
        } else if let Some(dist) = metric.distribution() {
            self.last_value.set_values(&dist.last_sequence);

            if let Some(mean_chart) = &mut self.mean_value {
                let mean_len = mean_chart.values.len() as f64;
                mean_chart.add_value((mean_len, dist.mean() as f64));
            } else {
                let mut new_inner =
                    ChartInner::with_capacity(self.last_value.capacity).with_title("μ (mean)");
                new_inner.add_value((0.0, dist.mean() as f64));
                self.mean_value = Some(new_inner);
            }
        }
    }

    pub fn min_y(&self) -> f64 {
        if let Some(mean) = &self.mean_value {
            self.last_value.min_y.min(mean.min_y)
        } else {
            self.last_value.min_y
        }
    }

    pub fn max_y(&self) -> f64 {
        if let Some(mean) = &self.mean_value {
            self.last_value.max_y.max(mean.max_y)
        } else {
            self.last_value.max_y
        }
    }

    pub fn min_x(&self) -> f64 {
        self.last_value
            .values
            .first()
            .map(|(x, _)| *x)
            .unwrap_or(0.0)
    }

    pub fn max_x(&self) -> f64 {
        self.last_value
            .values
            .last()
            .map(|(x, _)| *x)
            .unwrap_or(0.0)
    }

    pub fn x_bounds(&self) -> (f64, f64) {
        (self.min_x(), self.max_x())
    }

    pub fn y_bounds(&self) -> (f64, f64) {
        (self.min_y(), self.max_y())
    }

    pub fn value_chart<'a>(&'a self) -> &'a ChartInner {
        &self.last_value
    }

    pub fn mean_chart<'a>(&'a self) -> Option<&'a ChartInner> {
        self.mean_value.as_ref()
    }
}

impl Widget for &ChartData {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if let Some(mean) = &self.mean_value {
            chart_widget(
                self.x_bounds(),
                self.y_bounds(),
                vec![&self.last_value, mean],
            )
        } else {
            chart_widget(self.x_bounds(), self.y_bounds(), vec![&self.last_value])
        }
        .render(area, buf)
    }
}

fn chart_widget<'a>(
    x_bounds: (f64, f64),
    y_bounds: (f64, f64),
    charts: Vec<&'a ChartInner>,
) -> ratatui::widgets::Chart<'a> {
    let (min_x, max_x) = x_bounds;
    let (min_y, max_y) = y_bounds;

    let mid_y = (min_y + max_y) / 2.0;

    let datasets = charts
        .iter()
        .map(|dim| {
            Dataset::default()
                .name(dim.title())
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(dim.color()))
                .graph_type(GraphType::Line)
                .data(dim.values())
        })
        .collect::<Vec<_>>();

    Chart::new(datasets)
        .block(Block::bordered().title(Line::from(format!(" {} ", charts[0].title())).centered()))
        .x_axis(
            Axis::default()
                .style(Style::default().gray())
                .bounds([min_x, max_x]),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().gray())
                .bounds([min_y, max_y])
                .labels(Line::from(vec![
                    format!("{:.2}", min_y).bold().into(),
                    format!("{:.2}", mid_y).into(),
                    format!("{:.2}", max_y).bold().into(),
                ])),
        )
}

#[cfg(test)]
mod tests {
    use crate::chart::ChartInner;

    #[test]
    fn it_works() {
        let mut chart = ChartInner::with_capacity(5);
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
