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
use ratatui::text::Span;
use ratatui::widgets::{Cell, Row, Table};

const HEADER: [&str; 4] = ["   Gen", "Score", "Δ", ""];

pub struct ImprovementLogWidget;

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
        let border_style = crate::styles::panel_block(focused);

        let table = Table::default()
            .block(border_style)
            .header(header_row(&HEADER))
            .rows(striped_rows(rows))
            .column_spacing(1)
            .style(Color::White)
            .row_highlight_style(Style::new().on_black().bold())
            .column_highlight_style(Color::Gray)
            .cell_highlight_style(Style::new().reversed().yellow())
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
