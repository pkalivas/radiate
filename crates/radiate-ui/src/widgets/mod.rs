use crate::defaults::{
    ALT_ROW_BG_COLOR, COLORS, DISTRIBUTION_HEADER_CELLS, NORMAL_ROW_BG, STAT_HEADER_CELLS,
    TIME_HEADER_CELLS,
};
use crate::state::{AppState, AppTableState, MetricsTab};
use radiate_engines::{Chromosome, MetricSet};
use radiate_engines::{
    Metric,
    stats::{fmt_duration, metric_tags},
};
use ratatui::buffer::Buffer;
use ratatui::widgets::{StatefulWidget, Tabs, Widget};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize, palette::material},
    text::Line,
    widgets::{Block, Cell, Row, Table},
};
use ratatui::{
    text::Span,
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};
use tui_piechart::{PieChart, PieSlice};

pub(crate) mod filter;
pub(crate) mod summary;

pub struct MetricsTabWidget<'a, C: Chromosome> {
    state: &'a mut AppState<C>,
}

impl<'a, C: Chromosome> MetricsTabWidget<'a, C> {
    pub(crate) fn new(state: &'a mut AppState<C>) -> Self {
        Self { state }
    }
}

impl<'a, C: Chromosome> Widget for &mut MetricsTabWidget<'a, C> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered();
        let inner = block.inner(area);
        block.render(area, buf);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // tabs
                Constraint::Fill(1),   // table
                Constraint::Length(1), // help text
            ])
            .split(inner);

        let titles = ["Time", "Stats", "Dist"]
            .into_iter()
            .map(|t| Span::styled(format!(" {t} "), Style::default().fg(Color::White)));

        let index = match self.state.metrics_tab {
            MetricsTab::Time => 0,
            MetricsTab::Stats => 1,
            MetricsTab::Distributions => 2,
        };

        Tabs::default()
            .titles(titles)
            .select(index)
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .bold()
            .render(chunks[0], buf);

        match self.state.metrics_tab {
            MetricsTab::Time => TimeTableWidget::new(&mut self.state).render(chunks[1], buf),
            MetricsTab::Stats => StatsTableWidget::new(&mut self.state).render(chunks[1], buf),
            MetricsTab::Distributions => {
                DistributionTableWidget::new(&mut self.state).render(chunks[1], buf)
            }
        }

        help_text_widget().render(chunks[2], buf);
    }
}

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
        let items = tagged_metrics(&self.state.metrics, self.state, metric_tags::TIME);

        let slices = items
            .iter()
            .enumerate()
            .filter_map(|(index, (name, m))| {
                m.time_statistic().map(|time| {
                    let total_ms = time.sum().as_millis() as f64;

                    const MAX_LABEL_LEN: usize = 12;
                    let mut label = (*name).to_string();
                    if label.len() > MAX_LABEL_LEN {
                        label.truncate(MAX_LABEL_LEN - 1);
                        label.push('…');
                    }

                    PieSlice::new(name, total_ms, COLORS[index % COLORS.len()])
                })
            })
            .collect::<Vec<_>>();

        let piechart = PieChart::new(slices)
            .show_legend(true)
            .show_percentages(true)
            .block(Block::bordered())
            .high_resolution(true);

        self.state.time_table.update_rows(&items);

        let table = Table::default()
            .block(Block::bordered())
            .header(header_row(&TIME_HEADER_CELLS))
            .rows(striped_rows(metric_to_time_rows(items.into_iter())))
            .row_highlight_style(Style::default().bg(material::LIGHT_BLUE.c800))
            .widths(&[
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ]);

        let [left, right] =
            Layout::horizontal([Constraint::Percentage(40), Constraint::Fill(1)]).areas(area);

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
        let items = tagged_metrics(&self.state.metrics, self.state, metric_tags::STATISTIC);

        self.state.stats_table.update_rows(&items);

        let table = Table::default()
            .block(Block::bordered())
            .header(header_row(&STAT_HEADER_CELLS))
            .rows(striped_rows(metrics_into_stat_rows(items.into_iter())))
            .row_highlight_style(Style::default().bg(material::LIGHT_BLUE.c800))
            .widths(&[
                Constraint::Length(22),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ]);

        let display_any_chart = self.state.display_any_mini_chart();

        if display_any_chart {
            let [top, bottom] =
                Layout::vertical([Constraint::Fill(3), Constraint::Length(7)]).areas(area);

            render_scrollable_table(buf, top, table, &mut self.state.stats_table);

            if self.state.stats_table.row_count > 0 && bottom.height > 3 {
                let maybe_chart = self
                    .state
                    .get_chart_by_key(self.state.stats_table.selected_metric.unwrap_or(""));
                if let Some(chart) = maybe_chart {
                    if self.state.display_mini_chart() && self.state.display_mini_chart_mean() {
                        chart.render(bottom, buf);
                    } else if self.state.display_mini_chart_mean() {
                        if let Some(mean_chart) = chart.mean_chart() {
                            mean_chart.render(bottom, buf);
                        }
                    } else {
                        chart.value_chart().render(bottom, buf);
                    }
                }
            }
        } else {
            render_scrollable_table(buf, area, table, &mut self.state.stats_table);
        }
    }
}

pub struct DistributionTableWidget<'a, C: Chromosome> {
    state: &'a mut AppState<C>,
}

impl<'a, C: Chromosome> DistributionTableWidget<'a, C> {
    pub fn new(state: &'a mut AppState<C>) -> Self {
        Self { state }
    }
}

impl<'a, C: Chromosome> Widget for DistributionTableWidget<'a, C> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items = tagged_metrics(&self.state.metrics, self.state, metric_tags::DISTRIBUTION);

        self.state.distribution_table.update_rows(&items);

        let table = Table::default()
            .block(Block::bordered())
            .header(header_row(&DISTRIBUTION_HEADER_CELLS))
            .rows(striped_rows(metrics_into_dist_rows(items.into_iter())))
            .row_highlight_style(Style::default().bg(material::LIGHT_BLUE.c800))
            .widths(&[
                Constraint::Length(22),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ]);

        let display_any_chart = self.state.display_any_mini_chart();

        if display_any_chart {
            let [top, bottom] =
                Layout::vertical([Constraint::Fill(3), Constraint::Length(15)]).areas(area);

            render_scrollable_table(buf, top, table, &mut self.state.distribution_table);

            if self.state.distribution_table.row_count > 0 && bottom.height > 3 {
                let maybe_chart = self
                    .state
                    .get_chart_by_key(self.state.distribution_table.selected_metric.unwrap_or(""));
                if let Some(chart) = maybe_chart {
                    if self.state.display_any_mini_chart() && self.state.display_mini_chart_mean() {
                        chart.render(bottom, buf);
                    } else if self.state.display_mini_chart_mean() {
                        if let Some(mean_chart) = chart.mean_chart() {
                            mean_chart.render(bottom, buf);
                        }
                    } else {
                        chart.value_chart().render(bottom, buf);
                    }
                }
            }
        } else {
            render_scrollable_table(buf, area, table, &mut self.state.distribution_table);
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
    tag: &'static str,
) -> Vec<(&'static str, &'a Metric)> {
    let mut items: Vec<_> = metrics
        .iter_tagged(tag)
        .filter(|(_, m)| state.metric_has_tags(m))
        .collect();
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
                Cell::from(name.to_string()),
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

fn metrics_into_dist_rows<'a>(
    metrics: impl Iterator<Item = (&'static str, &'a Metric)>,
) -> impl Iterator<Item = Row<'a>> {
    metrics.filter_map(|(name, m)| {
        if let Some(dist) = m.distribution() {
            Some(Row::new(vec![
                Cell::from(name.to_string()),
                Cell::from(format!("{:.3}", dist.percentile(0.0))),
                Cell::from(format!("{:.3}", dist.percentile(25.0))),
                Cell::from(format!("{:.3}", dist.percentile(50.0))),
                Cell::from(format!("{:.3}", dist.percentile(75.0))),
                Cell::from(format!("{:.3}", dist.percentile(100.0))),
                Cell::from(format!("{}", dist.count())),
                Cell::from(format!("{:.2}", dist.standard_deviation())),
                Cell::from(format!("{:.2}", dist.variance())),
                Cell::from(format!("{:.2}", dist.skewness())),
                Cell::from(format!("{:.2}", dist.entropy())),
            ]))
        } else {
            None
        }
    })
}

pub fn striped_rows<'a>(rows: impl IntoIterator<Item = Row<'a>>) -> impl Iterator<Item = Row<'a>> {
    rows.into_iter().enumerate().map(|(i, row)| {
        let bg = if i % 2 == 0 {
            NORMAL_ROW_BG
        } else {
            ALT_ROW_BG_COLOR
        };
        row.style(Style::default().bg(bg))
    })
}

pub fn header_row<'a>(cols: &'a [&str]) -> Row<'a> {
    Row::new(cols.iter().copied().map(Cell::from))
        .height(1)
        .style(Style::default().bold().underlined().fg(Color::White))
}

pub fn help_text_widget() -> Paragraph<'static> {
    Paragraph::new(
        Line::from(vec![
            Span::from("Use "),
            "[j/k]".fg(Color::LightGreen).bold(),
            Span::from(" to navigate, "),
            "[t]".fg(Color::LightGreen).bold(),
            Span::from(" for Time, "),
            "[s]".fg(Color::LightGreen).bold(),
            Span::from(" for Stats, "),
            "[d]".fg(Color::LightGreen).bold(),
            Span::from(" for Distributions, "),
            "[f]".fg(Color::LightGreen).bold(),
            Span::from(" to toggle filters."),
            "[c]".fg(Color::LightGreen).bold(),
            Span::from(" to toggle rolling."),
            "[m]".fg(Color::LightGreen).bold(),
            Span::from(" to toggle means."),
        ])
        .centered(),
    )
}
