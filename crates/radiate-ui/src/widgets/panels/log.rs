use crate::state::{AppState, Pane};
use crate::widgets::AppWidget;
use radiate_engines::{Chromosome, Objective, Optimize};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Cell, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Table, Widget,
};

const HEADER: [&str; 4] = ["   Gen", "Score", "Δ", ""];

pub struct ImprovementLogWidget;

impl<C: Chromosome> AppWidget<C> for ImprovementLogWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppState<C>) {
        let log = &state.evo.improvement_log;
        let count = log.len();

        let max_delta = log.iter().map(|e| e.delta).fold(0.0_f32, f32::max);
        let best = state.evo.best_score.as_f32();
        let is_minimize = matches!(
            &state.evo.pareto.objective,
            Objective::Single(Optimize::Minimize)
        );

        let rows: Vec<Row> = log
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let bar = delta_bar(entry.delta, max_delta, 10);
                let bar_color = delta_color(entry.delta, max_delta);
                let score_color = score_quality_color(entry.score, best, is_minimize);
                let bg = crate::styles::alternating_row_style(i)
                    .bg
                    .unwrap_or(crate::styles::BG_COLOR);
                Row::new(vec![
                    Cell::from(format!("{:>6}", entry.generation))
                        .style(Style::default().fg(Color::Gray)),
                    Cell::from(format!("{:.6}", entry.score))
                        .style(Style::default().fg(score_color)),
                    Cell::from(Span::styled(
                        format!("+{:.6}", entry.delta),
                        Style::default().fg(Color::LightGreen),
                    )),
                    Cell::from(bar).style(Style::default().fg(bar_color)),
                ])
                .style(Style::default().bg(bg))
            })
            .collect();

        let focused = state.nav.is_pane_focused(Pane::List);
        let title = Line::from(vec![
            Span::raw(" Events "),
            Span::styled(format!("({count})"), Style::default().fg(Color::DarkGray)),
            Span::raw(" "),
        ]);

        let block = crate::styles::panel_block(focused).title(title);

        let header = Row::new(HEADER.iter().copied().map(Cell::from))
            .style(Style::default().bold().underlined().fg(Color::White));

        let table = Table::default()
            .block(block)
            .header(header)
            .rows(rows)
            .widths([
                Constraint::Length(8),
                Constraint::Length(14),
                Constraint::Length(14),
                Constraint::Fill(1),
            ]);

        let [tbl, scroll] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        Widget::render(table, tbl, buf);

        if count > tbl.height.saturating_sub(2) as usize {
            let mut scroll_state = ScrollbarState::new(count);
            StatefulWidget::render(
                Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .track_style(Style::default().fg(Color::DarkGray))
                    .thumb_style(Style::default().fg(Color::LightGreen)),
                scroll,
                buf,
                &mut scroll_state,
            );
        }
    }
}

/// Color based on how close `score` is to `best` — green at peak, dimmer as it falls away.
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

fn delta_bar(delta: f32, max_delta: f32, max_width: usize) -> String {
    if max_delta <= 0.0 {
        return String::new();
    }
    let filled = ((delta / max_delta).clamp(0.0, 1.0) * max_width as f32).round() as usize;
    "█".repeat(filled)
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
