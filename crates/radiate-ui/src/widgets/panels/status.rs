use crate::state::AppState;
use crate::widgets::panels::MetricLineChartWidget;
use crate::widgets::{AppWidget, FnWidget, MetricDetailPanelWidget, Panel, TabComponent};
use radiate_engines::stats::fmt_duration;
use radiate_engines::{Chromosome, MetricSet};
use ratatui::prelude::*;
use ratatui::style::{Color, Stylize};
use ratatui::widgets::{Paragraph, Row, Table};

pub struct EngineStatusPanelWidget;

impl EngineStatusPanelWidget {
    pub fn new() -> Self {
        Self
    }
}

impl<C: Chromosome> AppWidget<C> for EngineStatusPanelWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppState<C>) {
        let metrics = &state.evo.metrics;
        let elapsed = metrics
            .time()
            .and_then(|m| m.times().map(|t| t.sum()))
            .map(fmt_duration)
            .unwrap_or_else(|| "00:00:00.000".to_string());

        let rows = if state.evo.pareto.objective.is_single() {
            get_single_objective_summaries(metrics)
        } else {
            get_multi_objective_summaries(metrics)
        };

        let mut title = vec![
            "Gen ".fg(Color::Gray).bold(),
            format!("{}", state.evo.index).fg(Color::LightGreen),
        ];

        if state.evo.pareto.objective.is_single() {
            title.push(" | Score ".fg(Color::Gray).bold());
            title.push(format!("{:.4} ", state.evo.score.as_f32()).fg(Color::LightGreen));
        } else {
            title.push(" | MOGA ".fg(Color::Gray).bold());
        }

        title.push("| Time ".fg(Color::Gray).bold());
        title.push(elapsed.clone().fg(Color::LightGreen));

        let engine_table = Table::default()
            .rows(crate::styles::striped_rows(rows))
            .widths([Constraint::Fill(1), Constraint::Fill(1)]);

        let engine_state = if state.run.engine {
            if state.run.paused {
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

pub struct MetricModalWidget;

impl<C: Chromosome> AppWidget<C> for MetricModalWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppState<C>) {
        let index = state.chart_view_index();
        let tab_labels = state
            .selected_metric_views()
            .iter()
            .map(|v| {
                Span::styled(
                    format!(" {} ", v.label()),
                    Style::default().fg(Color::White),
                )
            })
            .collect::<Vec<Span<'static>>>();

        let [left, right] =
            Layout::horizontal([Constraint::Percentage(25), Constraint::Fill(1)]).areas(area);

        MetricDetailPanelWidget.render(left, buf, state);

        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Fill(1)])
            .split(right);

        Panel::new(FnWidget::new(move |area, buf| {
            TabComponent::new(tab_labels)
                .select(index)
                .render(area, buf);
        }))
        .render_inside_block(true)
        .render(areas[0], buf);

        MetricLineChartWidget::default()
            .with_show_bottom_options(false)
            .with_show_x_axis(true)
            .render(areas[1], buf, state);
    }
}

fn get_multi_objective_summaries(metrics: &MetricSet) -> Vec<Row<'static>> {
    let diversity = metrics.diversity_ratio().map(|m| m.mean()).unwrap_or(0.0);
    let carryover = metrics.carryover_rate().map(|m| m.mean()).unwrap_or(0.0);
    let unique_members = metrics.unique_members().map(|m| m.mean()).unwrap_or(0.0);
    let improvements = metrics.improvements().map(|m| m.count()).unwrap_or(0);
    let survivor_count = metrics.survivor_count().map(|m| m.mean()).unwrap_or(0.0);
    let new_children = metrics.new_children().map(|m| m.mean()).unwrap_or(0.0);
    let front_size = metrics.front_size().map(|m| m.mean()).unwrap_or(0.0);
    let front_entropy = metrics.front_entropy().map(|m| m.mean()).unwrap_or(0.0);
    let metric_meta = metrics.summary();

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
        Row::new(vec![
            "Metrics".bold(),
            format!("{}", metric_meta.metrics).into(),
        ]),
        Row::new(vec![
            "Updates".bold(),
            format_thousands(metric_meta.updates as usize)
                .to_string()
                .into(),
        ]),
    ];

    rows
}

fn get_single_objective_summaries(metrics: &MetricSet) -> Vec<Row<'static>> {
    let diversity = metrics.diversity_ratio().map(|m| m.mean()).unwrap_or(0.0);
    let carryover = metrics.carryover_rate().map(|m| m.mean()).unwrap_or(0.0);
    let unique_members = metrics.unique_members().map(|m| m.mean()).unwrap_or(0.0);
    let unique_scores = metrics.unique_scores().map(|m| m.mean()).unwrap_or(0.0);
    let improvements = metrics.improvements().map(|m| m.count()).unwrap_or(0);
    let survivor_count = metrics.survivor_count().map(|m| m.mean()).unwrap_or(0.0);
    let new_children = metrics.new_children().map(|m| m.mean()).unwrap_or(0.0);
    let metric_meta = metrics.summary();

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
        Row::new(vec![
            "Metrics".bold(),
            format!("{}", metric_meta.metrics).into(),
        ]),
        Row::new(vec![
            "Updates".bold(),
            format_thousands(metric_meta.updates as usize)
                .to_string()
                .into(),
        ]),
    ];

    rows
}

fn format_thousands(n: usize) -> String {
    n.to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",")
}
