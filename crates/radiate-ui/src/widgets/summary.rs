use crate::state::{AppState, ChartType};
use crate::styles::{ALT_BG_COLOR, BG_COLOR};
use crate::widgets::{ChartWidget, FnWidget, Panel};
use radiate_engines::stats::{TagType, fmt_duration};
use radiate_engines::{Chromosome, Metric, MetricSet};
use ratatui::prelude::*;
use ratatui::style::{Color, Stylize};
use ratatui::text::ToSpan;
use ratatui::widgets::{Paragraph, Row, Table, Tabs};

pub struct EngineSummaryWidget<C: Chromosome> {
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Chromosome> EngineSummaryWidget<C> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<C: Chromosome> StatefulWidget for EngineSummaryWidget<C> {
    type State = AppState<C>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let metrics = state.metrics();
        let elapsed = metrics
            .time()
            .and_then(|m| m.times().and_then(|t| t.sum()))
            .map(fmt_duration)
            .unwrap_or_else(|| "00:00:00.000".to_string());

        let rows = if state.objective_state.objective.is_single() {
            get_single_objective_summaries::<C>(&metrics)
        } else {
            get_multi_objective_summaries::<C>(&metrics)
        };

        let mut title = vec![
            "Gen ".fg(Color::Gray).bold(),
            format!("{}", state.index()).fg(Color::LightGreen),
        ];

        if state.objective_state.objective.is_single() {
            title.push(" | Score ".fg(Color::Gray).bold());
            title.push(format!("{:.4} ", state.score().as_f32()).fg(Color::LightGreen));
        } else {
            title.push(" | MOGA ".fg(Color::Gray).bold());
        }

        title.push("| Time ".fg(Color::Gray).bold());
        title.push(elapsed.clone().fg(Color::LightGreen));

        let engine_table = Table::default()
            .rows(striped_rows(rows))
            .widths(&[Constraint::Fill(1), Constraint::Fill(1)]);

        let engine_state = if state.is_engine_running() {
            if state.is_engine_paused() {
                " Paused ".fg(Color::Yellow).bold()
            } else {
                " Running ".fg(Color::LightGreen).bold()
            }
        } else {
            " Complete ".fg(Color::Red).bold()
        };

        Panel::new(FnWidget::new(|area, buf| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(15), Constraint::Fill(1)])
                .split(area);

            Paragraph::new(Line::from(title).centered()).render(layout[0], buf);
            Widget::render(engine_table, layout[1], buf);
        }))
        .titled(Line::from(engine_state).alignment(Alignment::Center))
        .render(area, buf);
    }
}

pub struct MetricSummaryWidget<'a, C: Chromosome> {
    _phantom: std::marker::PhantomData<C>,
    state: &'a AppState<C>,
}

impl<'a, C: Chromosome> MetricSummaryWidget<'a, C> {
    pub fn new(state: &'a AppState<C>) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            state,
        }
    }
}

impl<'a, C: Chromosome> Widget for MetricSummaryWidget<'a, C> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let current_metric_name = self.state.get_selected_metric().unwrap_or("");
        let metrics = self.state.metrics();
        let metric = metrics.get(current_metric_name);

        let Some(metric) = metric else {
            Paragraph::new(Line::from("No metric selected").centered()).render(area, buf);
            return;
        };

        let titles = ChartType::chart_options()
            .into_iter()
            .map(|t| Span::styled(format!(" {t} "), Style::default().fg(Color::White)));

        let index = match self.state.display.chart_id {
            ChartType::Value => 0,
            ChartType::Mean => 1,
            ChartType::Stddev => 2,
            ChartType::Variance => 3,
        };

        let [left, right] =
            Layout::horizontal([Constraint::Percentage(25), Constraint::Fill(1)]).areas(area);

        let chart_type = self.state.display.chart_id;
        let rows = if metric.tags().has(TagType::Statistic) {
            map_to_stat_metric_rows(metric)
        } else if metric.tags().has(TagType::Time) {
            map_to_time_metric_rows(metric)
        } else if metric.tags().has(TagType::Distribution) {
            map_to_distribution_metric_rows(metric)
        } else {
            vec![
                Row::new(vec!["Key".bold(), current_metric_name.into()]),
                Row::new(vec!["Type".bold(), "Unknown".into()]),
            ]
        };

        let metric_tags = metric
            .tags()
            .iter()
            .map(|t| Row::new(vec![format!("{t:?}").bold()]))
            .collect::<Vec<_>>();

        let tag_table = Table::default()
            .header(Row::new(vec![
                "Tags".to_span().bold().fg(crate::styles::SELECTED_GREEN),
            ]))
            .rows(striped_rows(metric_tags))
            .widths(&[Constraint::Fill(1)]);

        let metric_table = Table::default()
            .rows(striped_rows(rows))
            .style(Style::default().fg(Color::White))
            .widths(&[Constraint::Fill(1), Constraint::Fill(1)]);

        Panel::new(FnWidget::new(|area, buf| {
            let left_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(5),
                    Constraint::Fill(1),
                    Constraint::Percentage(10),
                ])
                .split(area);

            Widget::render(metric_table, left_layout[1], buf);
            Widget::render(tag_table, left_layout[2], buf);
        }))
        .titled(
            format!(" {} ", current_metric_name)
                .fg(crate::styles::SELECTED_GREEN)
                .bold(),
        )
        .render(left, buf);

        let charts = self.state.get_chart_by_key(current_metric_name, chart_type);

        Panel::new(FnWidget::new(|area, buf| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Fill(1)])
                .split(area);

            Tabs::new(titles)
                .select(index)
                .padding(" ", " ")
                .divider(" ")
                .highlight_style(crate::styles::selected_item_style())
                .bold()
                .render(chunks[0], buf);

            Panel::new(FnWidget::new(|area, buf| {
                ChartWidget::from(charts).render(area, buf);
            }))
            .render(chunks[1], buf);
        }))
        .titled(" Charts ")
        .render(right, buf);
    }
}

fn map_to_stat_metric_rows(metric: &Metric) -> Vec<Row<'_>> {
    if let Some(view) = metric.stats() {
        let rows = vec![
            Row::new(vec!["Type".bold(), metric.dtype().to_string().into()]),
            Row::new(vec!["Version".bold(), metric.version().to_string().into()]),
            Row::new(vec![
                "Updates".bold(),
                metric.update_count().to_string().into(),
            ]),
            Row::new(vec![
                "Last Value".bold(),
                format!("{:.2}", view.last().unwrap_or_default()).into(),
            ]),
            Row::new(vec![
                "Sum".bold(),
                format!("{:.4}", view.sum().unwrap_or_default()).into(),
            ]),
            Row::new(vec![
                "Min.".bold(),
                format!("{:.2}", view.min().unwrap_or_default()).into(),
            ]),
            Row::new(vec![
                "Max.".bold(),
                format!("{:.2}", view.max().unwrap_or_default()).into(),
            ]),
            Row::new(vec![
                "Mean".bold(),
                format!("{:.4}", view.mean().unwrap_or_default()).into(),
            ]),
            Row::new(vec![
                "Std Dev".bold(),
                format!("{:.4}", view.stddev().unwrap_or_default()).into(),
            ]),
            Row::new(vec![
                "Variance".bold(),
                format!("{:.4}", view.var().unwrap_or_default()).into(),
            ]),
        ];

        return rows;
    }

    return vec![];
}

fn map_to_time_metric_rows(metric: &Metric) -> Vec<Row<'_>> {
    if let Some(view) = metric.times() {
        let rows = vec![
            Row::new(vec!["Type".bold(), metric.dtype().to_string().into()]),
            Row::new(vec!["Version".bold(), metric.version().to_string().into()]),
            Row::new(vec![
                "Updates".bold(),
                metric.update_count().to_string().into(),
            ]),
            Row::new(vec![
                "Last Value".bold(),
                fmt_duration(view.last().unwrap_or_default()).into(),
            ]),
            Row::new(vec![
                "Sum".bold(),
                fmt_duration(view.sum().unwrap_or_default()).into(),
            ]),
            Row::new(vec![
                "Min.".bold(),
                fmt_duration(view.min().unwrap_or_default()).into(),
            ]),
            Row::new(vec![
                "Max.".bold(),
                fmt_duration(view.max().unwrap_or_default()).into(),
            ]),
            Row::new(vec![
                "Mean".bold(),
                fmt_duration(view.mean().unwrap_or_default()).into(),
            ]),
            Row::new(vec![
                "Std Dev".bold(),
                fmt_duration(view.stddev().unwrap_or_default()).into(),
            ]),
            Row::new(vec![
                "Variance".bold(),
                fmt_duration(view.var().unwrap_or_default()).into(),
            ]),
        ];

        return rows;
    }

    return vec![];
}

fn map_to_distribution_metric_rows(metric: &Metric) -> Vec<Row<'_>> {
    if let Some(view) = metric.distributions() {
        let rows = vec![
            Row::new(vec!["Type".bold(), metric.dtype().to_string().into()]),
            Row::new(vec!["Version".bold(), metric.version().to_string().into()]),
            Row::new(vec![
                "Updates".bold(),
                metric.update_count().to_string().into(),
            ]),
            Row::new(vec!["Count".bold(), view.count().to_string().into()]),
            Row::new(vec![
                "Sum".bold(),
                format!("{:.4}", view.sum().unwrap_or_default()).into(),
            ]),
            Row::new(vec![
                "Min.".bold(),
                format!("{:.2}", view.min().unwrap_or_default()).into(),
            ]),
            Row::new(vec![
                "Max.".bold(),
                format!("{:.2}", view.max().unwrap_or_default()).into(),
            ]),
            Row::new(vec![
                "Mean".bold(),
                format!("{:.4}", view.mean().unwrap_or_default()).into(),
            ]),
            Row::new(vec![
                "Std Dev".bold(),
                format!("{:.4}", view.stddev().unwrap_or_default()).into(),
            ]),
            Row::new(vec![
                "Variance".bold(),
                format!("{:.4}", view.var().unwrap_or_default()).into(),
            ]),
        ];

        return rows;
    }

    return vec![];
}

fn striped_rows<'a>(rows: impl IntoIterator<Item = Row<'a>>) -> impl Iterator<Item = Row<'a>> {
    rows.into_iter().enumerate().map(|(i, row)| {
        let bg = if i % 2 == 0 { BG_COLOR } else { ALT_BG_COLOR };
        row.style(Style::default().bg(bg))
    })
}

fn get_multi_objective_summaries<C: Chromosome>(metrics: &MetricSet) -> Vec<Row<'static>> {
    let diversity = metrics.diversity_ratio().map(|m| m.mean()).unwrap_or(0.0);
    let carryover = metrics.carryover_rate().map(|m| m.mean()).unwrap_or(0.0);
    let unique_members = metrics.unique_members().map(|m| m.mean()).unwrap_or(0.0);
    let improvements = metrics.improvements().map(|m| m.count()).unwrap_or(0);
    let survivor_count = metrics.survivor_count().map(|m| m.mean()).unwrap_or(0.0);
    let new_children = metrics.new_children().map(|m| m.mean()).unwrap_or(0.0);
    let front_size = metrics.front_size().map(|m| m.mean()).unwrap_or(0.0);
    let front_entropy = metrics.front_entropy().map(|m| m.mean()).unwrap_or(0.0);

    let rows = vec![
        Row::new(vec!["Improvements".bold(), improvements.to_string().into()]),
        Row::new(vec![
            "Diversity".bold(),
            format!("{:.2}%", diversity * 100.0).into(),
        ]),
        Row::new(vec![
            "Carryover".bold(),
            format!("{:.2}%", carryover * 100.0).into(),
        ]),
        Row::new(vec![
            "Unique Members".bold(),
            format!("{:.2}", unique_members).into(),
        ]),
        Row::new(vec![
            "Front Size".bold(),
            format!("{:.2}", front_size).into(),
        ]),
        Row::new(vec![
            "Front Entropy".bold(),
            format!("{:.4}", front_entropy).into(),
        ]),
        Row::new(vec![
            "Survivor / Gen.".bold(),
            format!("{:.2}", survivor_count).into(),
        ]),
        Row::new(vec![
            "Children / Gen.".bold(),
            format!("{:.2}", new_children).into(),
        ]),
    ];

    rows
}

fn get_single_objective_summaries<C: Chromosome>(metrics: &MetricSet) -> Vec<Row<'static>> {
    let diversity = metrics.diversity_ratio().map(|m| m.mean()).unwrap_or(0.0);
    let carryover = metrics.carryover_rate().map(|m| m.mean()).unwrap_or(0.0);
    let unique_members = metrics.unique_members().map(|m| m.mean()).unwrap_or(0.0);
    let unique_scores = metrics.unique_scores().map(|m| m.mean()).unwrap_or(0.0);
    let improvements = metrics.improvements().map(|m| m.count()).unwrap_or(0);
    let survivor_count = metrics.survivor_count().map(|m| m.mean()).unwrap_or(0.0);
    let new_children = metrics.new_children().map(|m| m.mean()).unwrap_or(0.0);

    let rows = vec![
        Row::new(vec!["Improvements".bold(), improvements.to_string().into()]),
        Row::new(vec![
            "Diversity".bold(),
            format!("{:.2}%", diversity * 100.0).into(),
        ]),
        Row::new(vec![
            "Carryover".bold(),
            format!("{:.2}%", carryover * 100.0).into(),
        ]),
        Row::new(vec![
            "Unique Members".bold(),
            format!("{:.2}", unique_members).into(),
        ]),
        Row::new(vec![
            "Unique Scores".bold(),
            format!("{:.2}", unique_scores).into(),
        ]),
        Row::new(vec![
            "Survivor / Gen.".bold(),
            format!("{:.2}", survivor_count).into(),
        ]),
        Row::new(vec![
            "Children / Gen.".bold(),
            format!("{:.2}", new_children).into(),
        ]),
    ];

    rows
}
