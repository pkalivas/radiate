use crate::chart::RollingChart;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    symbols,
    text::Line,
    widgets::{Axis, Block, Chart, Dataset, GraphType, Widget},
};
pub struct ChartWidget<'a> {
    charts: Vec<&'a RollingChart>,
}

impl<'a> ChartWidget<'a> {
    pub fn new(charts: Vec<&'a RollingChart>) -> Self {
        Self { charts }
    }
}

impl<'a> From<Vec<&'a RollingChart>> for ChartWidget<'a> {
    fn from(value: Vec<&'a RollingChart>) -> Self {
        Self::new(value)
    }
}

impl<'a> From<&'a RollingChart> for ChartWidget<'a> {
    fn from(value: &'a RollingChart) -> Self {
        Self::new(vec![value])
    }
}

impl<'a> From<Option<&'a RollingChart>> for ChartWidget<'a> {
    fn from(value: Option<&'a RollingChart>) -> Self {
        match value {
            Some(chart) => Self::new(vec![chart]),
            None => Self::new(vec![]),
        }
    }
}

impl<'a> From<Vec<Option<&'a RollingChart>>> for ChartWidget<'a> {
    fn from(value: Vec<Option<&'a RollingChart>>) -> Self {
        let charts = value.into_iter().filter_map(|c| c).collect();
        Self::new(charts)
    }
}

impl<'a> Widget for ChartWidget<'a> {
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

        let chart = chart_widget(x_bounds, y_bounds, self.charts);
        chart.render(area, buf);
    }
}

fn chart_widget<'a>(
    x_bounds: (f64, f64),
    y_bounds: (f64, f64),
    charts: Vec<&'a RollingChart>,
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
        .bg(crate::styles::ALT_BG_COLOR)
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
