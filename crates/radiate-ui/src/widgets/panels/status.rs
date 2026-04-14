use crate::state::{AppState, LineChartType};
use crate::widgets::{FnWidget, LineChartWidget, MetricDetailPanelWidget, Panel};
use radiate_engines::stats::fmt_duration;
use radiate_engines::{Chromosome, MetricSet};
use ratatui::prelude::*;
use ratatui::style::{Color, Stylize};
use ratatui::widgets::{Paragraph, Row, Table, Tabs};

pub struct EngineStatusPanelWidget<C: Chromosome> {
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Chromosome> EngineStatusPanelWidget<C> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<C: Chromosome> StatefulWidget for EngineStatusPanelWidget<C> {
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
            .rows(crate::styles::striped_rows(rows))
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

pub struct MetricModalWidget<C: Chromosome> {
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Chromosome> MetricModalWidget<C> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<C: Chromosome> StatefulWidget for MetricModalWidget<C> {
    type State = AppState<C>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let current_metric_name = state.get_selected_metric().unwrap_or("");

        let titles = LineChartType::chart_options()
            .into_iter()
            .map(|t| Span::styled(format!(" {t} "), Style::default().fg(Color::White)));

        let index = match state.display.chart_id {
            LineChartType::Value => 0,
            LineChartType::Mean => 1,
            LineChartType::Stddev => 2,
            LineChartType::Variance => 3,
        };

        let [left, right] =
            Layout::horizontal([Constraint::Percentage(25), Constraint::Fill(1)]).areas(area);

        MetricDetailPanelWidget::new().render(left, buf, state);

        let chart_type = state.display.chart_id;
        let charts = state.get_chart_by_key(current_metric_name, chart_type);

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
                LineChartWidget::from(charts)
                    .with_show_x_axis(true)
                    .render(area, buf);
            }))
            .render(chunks[1], buf);
        }))
        .titled(" Charts ")
        .render(right, buf);
    }
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
