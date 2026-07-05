use super::tables::{header_row, render_scrollable_table, striped_rows};
use crate::widgets::AppWidget;
use crate::{
    state::{AppState, Pane},
    styles::delta_bar,
};
use radiate_engines::{Chromosome, Objective, Optimize};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Bar, BarChart, BarGroup, Block, Cell, Row, Table, Widget};

const IMPROVEMENT_HEADER: [&str; 4] = ["   Gen", "Score", "Δ", ""];
const FRONT_EVENT_HEADER: [&str; 6] = [
    "   Gen", "Size", "+Added", "-Removed", "Compared", "Filtered",
];

pub struct ImprovementLogWidget;
pub struct FrontEventLogWidget;

impl<C: Chromosome> AppWidget<C> for ImprovementLogWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppState<C>) {
        let log = &state.evo.improvement_log;

        let max_delta = log.iter().map(|e| e.delta).fold(0.0_f32, f32::max);
        let best = state.evo.best_score.as_f32();
        let is_minimize = matches!(
            &state.evo.pareto.objective,
            Objective::Single(Optimize::Minimize)
        );

        state
            .tables
            .log
            .update_rows(log.as_slice(), |entry| entry.generation);

        let rows = log
            .iter()
            .map(|entry| {
                let bar = delta_bar(entry.delta, max_delta, 10);
                let bar_color = delta_color(entry.delta, max_delta);
                let score_color = score_quality_color(entry.score, best, is_minimize);

                Row::new(vec![
                    Cell::from(format!("{:>6}", entry.generation))
                        .style(Style::default().fg(crate::styles::TEXT_FG_COLOR)),
                    Cell::from(format!("{:.6}", entry.score))
                        .style(Style::default().fg(score_color)),
                    Cell::from(Span::styled(
                        format!("+{:.6}", entry.delta),
                        Style::default().fg(crate::styles::TREND_UP_COLOR),
                    )),
                    Cell::from(bar).style(Style::default().fg(bar_color)),
                ])
            })
            .collect::<Vec<_>>();

        let focused = state.nav.is_pane_focused(Pane::List);
        let table = Table::default()
            .block(crate::styles::panel_block(focused))
            .header(header_row(&IMPROVEMENT_HEADER))
            .rows(striped_rows(rows))
            .column_spacing(1)
            .style(Color::White)
            .row_highlight_style(Style::new().on_black().bold())
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
            .highlight_symbol(Span::styled("▶ ", Style::default().fg(Color::LightGreen)))
            .widths([
                Constraint::Length(8),
                Constraint::Length(14),
                Constraint::Length(14),
                Constraint::Fill(1),
            ]);

        render_scrollable_table(buf, area, table, &mut state.tables.log);
    }
}

impl<C: Chromosome> AppWidget<C> for FrontEventLogWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppState<C>) {
        let log = &state.evo.front_event_log;

        let max_additions = log.iter().map(|e| e.additions).fold(0, usize::max);
        let max_removals = log.iter().map(|e| e.removals).fold(0, usize::max);
        let max_comparisons = log.iter().map(|e| e.comparisons).fold(0, usize::max);

        state
            .tables
            .log
            .update_rows(log.as_slice(), |entry| entry.generation);

        let rows = log
            .iter()
            .map(|entry| {
                let add_ratio = if max_additions > 0 {
                    entry.additions as f32 / max_additions as f32
                } else {
                    0.0
                };
                let rem_ratio = if max_removals > 0 {
                    entry.removals as f32 / max_removals as f32
                } else {
                    0.0
                };
                let comp_ratio = if max_comparisons > 0 {
                    entry.comparisons as f32 / max_comparisons as f32
                } else {
                    0.0
                };

                Row::new(vec![
                    Cell::from(format!("{:>6}", entry.generation))
                        .style(Style::default().fg(crate::styles::TEXT_FG_COLOR)),
                    Cell::from(format!("{}", entry.front_size))
                        .style(Style::default().fg(crate::styles::TEXT_FG_COLOR)),
                    Cell::from(Line::from(vec![
                        Span::styled("+", Style::default().fg(crate::styles::TREND_UP_COLOR)),
                        Span::styled(
                            format!("{}", entry.additions),
                            Style::default()
                                .fg(crate::styles::sentiment_color(add_ratio, 0.2, 0.6)),
                        ),
                    ])),
                    Cell::from(Line::from(vec![
                        Span::styled("-", Style::default().fg(crate::styles::TREND_DOWN_COLOR)),
                        Span::styled(
                            format!("{}", entry.removals),
                            Style::default().fg(if entry.removals == 0 {
                                Color::DarkGray
                            } else {
                                crate::styles::sentiment_color(1.0 - rem_ratio, 0.2, 0.6)
                            }),
                        ),
                    ])),
                    Cell::from(Line::from(vec![
                        Span::styled("", Style::default().fg(crate::styles::TREND_NEUTRAL_COLOR)),
                        Span::styled(
                            crate::styles::format_thousands(entry.comparisons),
                            Style::default()
                                .fg(crate::styles::sentiment_color(comp_ratio, 0.2, 0.6)),
                        ),
                    ])),
                    Cell::from(if entry.filtered {
                        Span::styled(
                            "Y",
                            Style::default().fg(crate::styles::TREND_UP_COLOR_LIGHT),
                        )
                    } else {
                        Span::styled(
                            "N",
                            Style::default().fg(crate::styles::TREND_DOWN_COLOR_LIGHT),
                        )
                    }),
                ])
            })
            .collect::<Vec<_>>();

        let focused = state.nav.is_pane_focused(Pane::List);
        let table = Table::default()
            .block(crate::styles::panel_block(focused))
            .header(header_row(&FRONT_EVENT_HEADER))
            .rows(striped_rows(rows))
            .column_spacing(1)
            .style(Color::White)
            .row_highlight_style(Style::new().on_black().bold())
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
            .highlight_symbol(Span::styled("▶ ", Style::default().fg(Color::LightGreen)))
            .widths([
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ]);

        render_scrollable_table(buf, area, table, &mut state.tables.log);
    }
}

pub struct DeltaBarChartWidget;

impl<C: Chromosome> AppWidget<C> for DeltaBarChartWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppState<C>) {
        let log = &state.evo.improvement_log;
        let max_delta = log.iter().map(|e| e.delta).fold(0.0_f32, f32::max);

        let entries = log.as_slice();
        let bars = entries
            .iter()
            .rev()
            .map(|entry| {
                let value = if max_delta > 0.0 {
                    ((entry.delta / max_delta) * 1000.0) as u64
                } else {
                    0
                };
                Bar::default()
                    .value(value)
                    .text_value(String::new())
                    .style(Style::default().fg(delta_color(entry.delta, max_delta)))
            })
            .collect::<Vec<_>>();

        let focused = state.nav.is_pane_focused(Pane::Chart);
        let border_color = if focused {
            crate::styles::BORDER_GREEN
        } else {
            Color::DarkGray
        };

        Widget::render(
            BarChart::default()
                .block(
                    Block::bordered()
                        .title(
                            Line::from(Span::styled(
                                format!(" Improvement Δ "),
                                Style::default().fg(Color::White).bold(),
                            ))
                            .centered(),
                        )
                        .border_style(Style::default().fg(border_color)),
                )
                .data(BarGroup::default().bars(&bars))
                .bar_width(1)
                .bar_gap(0)
                .max(1000),
            area,
            buf,
        );
    }
}

fn score_quality_color(score: f32, best: f32, is_minimize: bool) -> Color {
    if best == 0.0 {
        return Color::White;
    }
    let ratio = if is_minimize {
        best / score.max(f32::EPSILON)
    } else {
        score / best
    }
    .clamp(0.0, 1.0);

    crate::styles::sentiment_color(ratio, 0.7, 0.95)
}

fn delta_color(delta: f32, max_delta: f32) -> Color {
    if max_delta <= 0.0 {
        return Color::DarkGray;
    }
    let ratio = (delta / max_delta).clamp(0.0, 1.0);
    if ratio >= 0.6 {
        Color::LightGreen
    } else if ratio >= 0.25 {
        Color::Green
    } else {
        Color::DarkGray
    }
}
