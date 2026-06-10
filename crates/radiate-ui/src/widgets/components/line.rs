use crate::chart::LineChart;
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
}

impl<'a, T: LineChart> LineChartWidget<'a, T> {
    pub fn new(charts: Vec<&'a T>) -> Self {
        Self {
            charts,
            bg_color: crate::styles::ALT_BG_COLOR,
            show_x_axis: false,
            show_boarders: true,
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

        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;
        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;

        for chart in &self.charts {
            if chart.min_y() < min_y {
                min_y = chart.min_y();
            }
            if chart.min_x() < min_x {
                min_x = chart.min_x();
            }
            if chart.max_y() > max_y {
                max_y = chart.max_y();
            }
            if chart.max_x() > max_x {
                max_x = chart.max_x();
            }
        }

        let x_bounds = (min_x, max_x);
        let y_bounds = (min_y, max_y);

        let chart = chart_widget(
            x_bounds,
            y_bounds,
            self.charts,
            self.bg_color,
            self.show_x_axis,
            self.show_boarders,
        );

        chart.render(area, buf);
    }
}

fn chart_widget<'a, T: LineChart>(
    x_bounds: (f64, f64),
    y_bounds: (f64, f64),
    charts: Vec<&'a T>,
    bg_color: Color,
    show_x_axis: bool,
    show_boarders: bool,
) -> ratatui::widgets::Chart<'a> {
    let (min_x, max_x) = x_bounds;
    let (min_y, max_y) = y_bounds;

    let mid_y = (min_y + max_y) / 2.0;
    let mid_x = (min_x + max_x) / 2.0;

    let datasets = charts
        .iter()
        .map(|dim| {
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(dim.color()))
                .graph_type(GraphType::Line)
                .data(dim.values())
        })
        .collect::<Vec<_>>();

    let x_axis = if show_x_axis {
        Axis::default()
            .style(Style::default().gray())
            .bounds([min_x, max_x])
            .labels(Line::from(vec![
                format!("{:.2}", min_x).bold(),
                format!("{:.2}", mid_x).into(),
                format!("{:.2}", max_x).bold(),
            ]))
    } else {
        Axis::default()
            .style(Style::default().gray())
            .bounds([min_x, max_x])
    };

    let result = Chart::new(datasets).bg(bg_color).x_axis(x_axis).y_axis(
        Axis::default()
            .style(Style::default().fg(bg_color))
            .bounds([min_y, max_y])
            .labels(Line::from(vec![
                format!("{:.2}", min_y).bold(),
                format!("{:.2}", mid_y).into(),
                format!("{:.2}", max_y).bold(),
            ])),
    );

    if show_boarders {
        result.block(
            Block::bordered().title(Line::from(format!(" {} ", charts[0].title())).centered()),
        )
    } else {
        result
    }
}
