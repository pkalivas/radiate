use crate::chart::{ChartView, LineChart};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    symbols,
    text::Line,
    widgets::{Axis, Block, Chart, Dataset, GraphType, Widget},
};

pub struct LineChartWidget<'a, T: LineChart> {
    charts: Vec<&'a T>,
    bg_color: Color,
    show_x_axis: bool,
    show_boarders: bool,
    view: ChartView,
}

impl<'a, T: LineChart> LineChartWidget<'a, T> {
    pub fn new(charts: Vec<&'a T>) -> Self {
        Self {
            charts,
            bg_color: crate::styles::ALT_BG_COLOR,
            show_x_axis: false,
            show_boarders: true,
            view: ChartView::default(),
        }
    }

    pub fn with_show_x_axis(mut self, show: bool) -> Self {
        self.show_x_axis = show;
        self
    }

    pub fn with_show_boarders(mut self, show: bool) -> Self {
        self.show_boarders = show;
        self
    }

    pub fn with_view(mut self, view: ChartView) -> Self {
        self.view = view;
        self
    }
}

impl<'a, T: LineChart> From<Vec<&'a T>> for LineChartWidget<'a, T> {
    fn from(value: Vec<&'a T>) -> Self {
        Self::new(value)
    }
}

impl<'a, T: LineChart> From<&'a T> for LineChartWidget<'a, T> {
    fn from(value: &'a T) -> Self {
        Self::new(vec![value])
    }
}

impl<'a, T: LineChart> From<Option<&'a T>> for LineChartWidget<'a, T> {
    fn from(value: Option<&'a T>) -> Self {
        match value {
            Some(chart) => Self::new(vec![chart]),
            None => Self::new(vec![]),
        }
    }
}

impl<'a, T: LineChart> From<Vec<Option<&'a T>>> for LineChartWidget<'a, T> {
    fn from(value: Vec<Option<&'a T>>) -> Self {
        let charts = value.into_iter().flatten().collect();
        Self::new(charts)
    }
}

impl<'a, T: LineChart> Widget for LineChartWidget<'a, T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.charts.is_empty() {
            let block = Block::bordered().title(Line::from(" No Data ").centered());
            block.render(area, buf);
            return;
        }

        // Bounds come from the points actually drawn, so a shorter history
        // or a log scale rescales the axes to fit.
        let visible = self
            .charts
            .iter()
            .map(|chart| self.view.visible(chart.values()))
            .collect::<Vec<_>>();
        let transform = self
            .view
            .transform(visible.iter().flat_map(|v| v.iter().map(|&(_, y)| y)));
        let series = visible
            .iter()
            .map(|points| {
                points
                    .iter()
                    .map(|&(x, y)| (x, transform.to_chart(y)))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let (mut min_x, mut max_x) = (f64::MAX, f64::MIN);
        let (mut min_y, mut max_y) = (f64::MAX, f64::MIN);
        for &(x, y) in series.iter().flatten() {
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }

        if min_x > max_x {
            (min_x, max_x, min_y, max_y) = (0.0, 1.0, 0.0, 1.0);
        }
        if min_y == max_y {
            // avoid zero range
            min_y -= 0.5;
            max_y += 0.5;
        }

        let datasets = self
            .charts
            .iter()
            .zip(&series)
            .map(|(chart, points)| {
                Dataset::default()
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(chart.color()))
                    .graph_type(GraphType::Line)
                    .data(points)
            })
            .collect::<Vec<_>>();

        let x_axis = Axis::default()
            .style(Style::default().gray())
            .bounds([min_x, max_x]);
        let x_axis = if self.show_x_axis {
            x_axis.labels(Line::from(vec![
                format!("{:.2}", min_x).bold(),
                format!("{:.2}", (min_x + max_x) / 2.0).into(),
                format!("{:.2}", max_x).bold(),
            ]))
        } else {
            x_axis
        };

        // Labels show real values, whatever the scale.
        let label = |y: f64| format_value(transform.to_value(y));
        let y_axis = Axis::default()
            .style(Style::default().fg(self.bg_color))
            .bounds([min_y, max_y])
            .labels(Line::from(vec![
                label(min_y).bold(),
                label((min_y + max_y) / 2.0).into(),
                label(max_y).bold(),
            ]));

        let chart = Chart::new(datasets)
            .bg(self.bg_color)
            .x_axis(x_axis)
            .y_axis(y_axis);

        if self.show_boarders {
            let title = format!(" {} ", self.charts[0].title());
            chart
                .block(Block::bordered().title(Line::from(title).centered()))
                .render(area, buf);
        } else {
            chart.render(area, buf);
        }
    }
}

/// Two decimals, switching to scientific notation where that would show
/// nothing useful (very large, or small but not zero).
fn format_value(v: f64) -> String {
    let abs = v.abs();
    if abs != 0.0 && !(0.01..1e6).contains(&abs) {
        format!("{v:.2e}")
    } else {
        format!("{v:.2}")
    }
}
