use crate::state::AppState;
use crate::styles::{ALT_BG_COLOR, BG_COLOR};
use crate::widgets::{FnWidget, Panel};
use radiate_engines::stats::fmt_duration;
use radiate_engines::{Chromosome, MetricSet};
use ratatui::prelude::*;
use ratatui::style::{Color, Stylize};
use ratatui::widgets::{Paragraph, Row, Table};

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
            .and_then(|m| m.time_sum())
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

fn striped_rows<'a>(rows: impl IntoIterator<Item = Row<'a>>) -> impl Iterator<Item = Row<'a>> {
    rows.into_iter().enumerate().map(|(i, row)| {
        let bg = if i % 2 == 0 { BG_COLOR } else { ALT_BG_COLOR };
        row.style(Style::default().bg(bg))
    })
}

fn get_multi_objective_summaries<C: Chromosome>(metrics: &MetricSet) -> Vec<Row<'static>> {
    let diversity = metrics
        .diversity_ratio()
        .and_then(|m| m.value_mean())
        .unwrap_or(0.0);
    let carryover = metrics
        .carryover_rate()
        .and_then(|m| m.value_mean())
        .unwrap_or(0.0);
    let unique_members = metrics
        .unique_members()
        .and_then(|m| m.value_mean())
        .unwrap_or(0.0);
    let improvements = metrics.improvements().map(|m| m.count()).unwrap_or(0);
    let survivor_count = metrics
        .survivor_count()
        .and_then(|m| m.value_mean())
        .unwrap_or(0.0);
    let new_children = metrics
        .new_children()
        .and_then(|m| m.value_mean())
        .unwrap_or(0.0);
    let front_size = metrics
        .front_size()
        .and_then(|m| m.value_mean())
        .unwrap_or(0.0);
    let front_entropy = metrics
        .front_entropy()
        .and_then(|m| m.value_mean())
        .unwrap_or(0.0);

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
    let diversity = metrics
        .diversity_ratio()
        .and_then(|m| m.value_mean())
        .unwrap_or(0.0);
    let carryover = metrics
        .carryover_rate()
        .and_then(|m| m.value_mean())
        .unwrap_or(0.0);
    let unique_members = metrics
        .unique_members()
        .and_then(|m| m.value_mean())
        .unwrap_or(0.0);
    let unique_scores = metrics
        .unique_scores()
        .and_then(|m| m.value_mean())
        .unwrap_or(0.0);
    let improvements = metrics.improvements().map(|m| m.count()).unwrap_or(0);
    let survivor_count = metrics
        .survivor_count()
        .and_then(|m| m.value_mean())
        .unwrap_or(0.0);
    let new_children = metrics
        .new_children()
        .and_then(|m| m.value_mean())
        .unwrap_or(0.0);

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
