use std::vec;

use crate::state::AppState;
use crate::widgets::modal::Modal;
use crate::widgets::{ChartWidget, Panel};
use crate::widgets::{
    MetricsTabWidget, ParetoPager, filter::FilterWidget, summary::EngineBaseWidget,
};
use radiate_engines::Chromosome;
use ratatui::layout::Alignment;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::Widget,
};

pub(crate) struct RootWidget<'a, C>
where
    C: Chromosome,
{
    state: &'a mut AppState<C>,
}

impl<'a, C> RootWidget<'a, C>
where
    C: Chromosome,
{
    pub fn new(state: &'a mut AppState<C>) -> Self {
        Self { state }
    }
}

impl<'a, C> Widget for &mut RootWidget<'a, C>
where
    C: Chromosome,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [top, bottom] =
            Layout::vertical([Constraint::Percentage(30), Constraint::Fill(1)]).areas(area);
        let [engine, fitness] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Fill(1)]).areas(top);

        EngineBaseWidget::new(&self.state).render(engine, buf);

        if self.state.objective_state.objective.is_single() {
            let chart_state = self.state.chart_state();
            let charts = if self.state.display_mini_chart_mean() {
                vec![
                    chart_state.fitness_chart(),
                    chart_state.fitness_mean_chart(),
                ]
            } else {
                vec![chart_state.fitness_chart()]
            };

            ChartWidget::from(charts).render(fitness, buf);
        } else {
            Panel::untitled(ParetoPager::new(&self.state)).render(fitness, buf);
        }

        if self.state.display.show_tag_filters {
            let [filter, tabs] =
                Layout::horizontal([Constraint::Length(20), Constraint::Fill(1)]).areas(bottom);
            FilterWidget::new(&mut self.state).render(filter, buf);
            MetricsTabWidget::new(&mut self.state).render(tabs, buf);
        } else {
            let [inner] = Layout::horizontal([Constraint::Fill(1)]).areas(bottom);
            MetricsTabWidget::new(&mut self.state).render(inner, buf);
        };

        if self.state.display.show_help {
            let body = Paragraph::new(help_text())
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true });

            let help_modal = Modal::new(body).title(" Help ").size_pct(70, 80);
            help_modal.render(area, buf);
        }
    }
}

fn help_text() -> Text<'static> {
    Text::from(vec![
        Line::from(vec![Span::styled(
            "Controls",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("General"),
        Line::from("  q           Quit UI"),
        Line::from("  ? / H       Toggle this help"),
        Line::from("  p           Pause / Resume engine"),
        Line::from("  n           Step one epoch (stays paused)"),
        Line::from(""),
        Line::from("Navigation"),
        Line::from("  j / Down    Move selection down"),
        Line::from("  k / Up      Move selection up"),
        Line::from("  h / Left    Previous metrics tab"),
        Line::from("  l / Right   Next metrics tab"),
        Line::from(""),
        Line::from("Charts / Objective pairs"),
        Line::from("  [ / ]       Prev / next objective-pair page"),
        Line::from("  + / -       Expand / shrink objective pairs"),
        Line::from("  c           Toggle mini chart"),
        Line::from("  m           Toggle mini chart mean"),
        Line::from(""),
        Line::from("Filters"),
        Line::from("  f           Toggle tag filters panel"),
        Line::from("  Enter       Toggle tag selection"),
        Line::from("  Esc         Clear tag filters"),
        Line::from("  0-9         Select filter by index"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press Esc to close",
            Style::default().add_modifier(Modifier::DIM),
        )]),
    ])
}
