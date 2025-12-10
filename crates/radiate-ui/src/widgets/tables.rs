use crate::state::{AppState, AppTableState, ChartType};
use crate::styles::{self, COLOR_WHEEL_400};
use crate::widgets::ChartWidget;
use radiate_engines::stats::TagKind;
use radiate_engines::{Chromosome, MetricSet, metric_names};
use radiate_engines::{Metric, stats::fmt_duration};
use ratatui::buffer::Buffer;
use ratatui::text::Line;
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::widgets::{StatefulWidget, Widget};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Cell, Row, Table},
};
use std::iter::{once, repeat};
use tui_piechart::{PieChart, PieSlice};

pub const STAT_HEADER_CELLS: [&str; 8] = [
    "Metric",
    "Min",
    "Max",
    "μ (mean)",
    "Sum",
    "StdDev",
    "Var",
    "Count",
];

pub const TIME_HEADER_CELLS: [&str; 5] = ["Metric", "Min", "Max", "μ (mean)", "Total"];

pub struct TimeTableWidget<'a, C: Chromosome> {
    state: &'a mut AppState<C>,
}

impl<'a, C: Chromosome> TimeTableWidget<'a, C> {
    pub fn new(state: &'a mut AppState<C>) -> Self {
        Self { state }
    }
}

impl<'a, C: Chromosome> Widget for TimeTableWidget<'a, C> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items = tagged_metrics(&self.state.metrics, self.state, TagKind::Time)
            .iter()
            .filter(|met| met.0 != metric_names::TIME)
            .map(|m| *m)
            .collect::<Vec<_>>();
        self.state.time_table.update_rows(&items);

        let slices = items
            .iter()
            .enumerate()
            .filter_map(|(index, (name, m))| {
                m.time_statistic().map(|time| {
                    let total_ms = time.sum().as_millis() as f64;

                    let color = if let Some(selected_name) = self.state.time_table.selected_metric {
                        if selected_name == *name {
                            COLOR_WHEEL_400[index % COLOR_WHEEL_400.len()]
                        } else {
                            Color::DarkGray
                        }
                    } else {
                        Color::DarkGray
                    };

                    PieSlice::new(name, total_ms, color)
                })
            })
            .collect::<Vec<_>>();

        let piechart = PieChart::new(slices)
            .show_legend(false)
            .show_percentages(true)
            .block(Block::bordered())
            .legend_layout(tui_piechart::LegendLayout::Horizontal)
            .high_resolution(true);

        let table = Table::default()
            .block(Block::bordered())
            .header(header_row(&TIME_HEADER_CELLS))
            .rows(striped_rows(metric_to_time_rows(items.into_iter())))
            .row_highlight_style(styles::selected_item_style())
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
            .widths(&[
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ]);

        let [left, right] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Fill(1)]).areas(area);

        piechart.render(left, buf);

        render_scrollable_table(buf, right, table, &mut self.state.time_table);
    }
}

pub struct StatsTableWidget<'a, C: Chromosome> {
    state: &'a mut AppState<C>,
}

impl<'a, C: Chromosome> StatsTableWidget<'a, C> {
    pub fn new(state: &'a mut AppState<C>) -> Self {
        Self { state }
    }
}

impl<'a, C: Chromosome> Widget for StatsTableWidget<'a, C> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items = tagged_metrics(&self.state.metrics, self.state, TagKind::Statistic);

        self.state.stats_table.update_rows(&items);

        let table = Table::default()
            .block(Block::bordered())
            .header(header_row(&STAT_HEADER_CELLS))
            .rows(striped_rows(metrics_into_stat_rows(items.into_iter())))
            .row_highlight_style(styles::selected_item_style())
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
            .widths(once(Constraint::Length(22)).chain(repeat(Constraint::Fill(1)).take(7)));

        let display_any_chart = self.state.display_any_mini_chart();

        if display_any_chart {
            let [top, bottom] =
                Layout::vertical([Constraint::Fill(3), Constraint::Length(7)]).areas(area);

            render_scrollable_table(buf, top, table, &mut self.state.stats_table);

            if self.state.stats_table.row_count > 0 && bottom.height > 3 {
                let selected_metric = self.state.stats_table.selected_metric.unwrap_or("");
                if self.state.display_mini_chart() && self.state.display_mini_chart_mean() {
                    let maybe_chart = self
                        .state
                        .get_chart_by_key(selected_metric, ChartType::Mean);
                    let maybe_value_chart = self
                        .state
                        .get_chart_by_key(selected_metric, ChartType::Value);

                    ChartWidget::from(vec![maybe_value_chart, maybe_chart]).render(bottom, buf);
                } else if self.state.display_mini_chart_mean() {
                    let maybe_chart = self
                        .state
                        .get_chart_by_key(selected_metric, ChartType::Mean);
                    ChartWidget::from(vec![maybe_chart]).render(bottom, buf);
                } else {
                    let maybe_chart = self
                        .state
                        .get_chart_by_key(selected_metric, ChartType::Value);
                    ChartWidget::from(vec![maybe_chart]).render(bottom, buf);
                }
            }
        } else {
            render_scrollable_table(buf, area, table, &mut self.state.stats_table);
        }
    }
}

fn render_scrollable_table(buf: &mut Buffer, area: Rect, table: Table, state: &mut AppTableState) {
    let [tbl, scroll] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

    StatefulWidget::render(&table, tbl, buf, &mut state.state);

    if state.row_count > tbl.height as usize {
        let mut scrollbar_state = state
            .scroll_bar
            .get_or_insert_with(|| ScrollbarState::new(state.row_count));

        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .track_style(Style::default().fg(Color::DarkGray))
            .thumb_style(Style::default().fg(Color::LightGreen));

        scrollbar.render(scroll, buf, &mut scrollbar_state);
    }
}

fn tagged_metrics<'a, C: Chromosome>(
    metrics: &'a MetricSet,
    state: &AppState<C>,
    tag: TagKind,
) -> Vec<(&'static str, &'a Metric)> {
    let mut items = metrics
        .iter_tagged(tag)
        .filter(|(_, m)| state.metric_has_tags(m))
        .collect::<Vec<_>>();
    items.sort_by(|a, b| a.0.cmp(b.0));
    items
}

/// --- Row Builders ---
fn metric_to_time_rows<'a>(
    metrics: impl Iterator<Item = (&'static str, &'a Metric)>,
) -> impl Iterator<Item = Row<'a>> {
    metrics.filter_map(|(name, m)| {
        if let Some(time) = m.time_statistic() {
            let mean = fmt_duration(time.mean());
            let min = fmt_duration(time.min());
            let max = fmt_duration(time.max());
            let total = fmt_duration(time.sum());

            Some(Row::new(vec![
                Cell::from(name.to_string()),
                Cell::from(min),
                Cell::from(max),
                Cell::from(mean),
                Cell::from(total),
            ]))
        } else {
            None
        }
    })
}

fn metrics_into_stat_rows<'a>(
    metrics: impl Iterator<Item = (&'static str, &'a Metric)>,
) -> impl Iterator<Item = Row<'a>> {
    metrics.filter_map(|(name, m)| {
        if let Some(stat) = m.statistic() {
            Some(Row::new(vec![
                Cell::from(Line::from(name.to_string())),
                Cell::from(format!("{:.2}", stat.min())),
                Cell::from(format!("{:.2}", stat.max())),
                Cell::from(format!("{:.2}", stat.mean())),
                Cell::from(format!("{:.2}", stat.sum())),
                Cell::from(format!("{:.2}", stat.std_dev())),
                Cell::from(format!("{:.2}", stat.variance())),
                Cell::from(format!("{}", stat.count())),
            ]))
        } else {
            None
        }
    })
}

fn striped_rows<'a>(rows: impl IntoIterator<Item = Row<'a>>) -> impl Iterator<Item = Row<'a>> {
    rows.into_iter()
        .enumerate()
        .map(|(i, row)| row.style(styles::alternating_row_style(i)))
}

fn header_row<'a>(cols: &'a [&str]) -> Row<'a> {
    Row::new(cols.iter().copied().map(Cell::from))
        .height(1)
        .style(Style::default().bold().underlined().fg(Color::White))
}
