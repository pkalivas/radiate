use crate::state::{AppState, MetricsTab};
use radiate_engines::Chromosome;
mod pareto;
pub use pareto::{ParetoFrontWidget, kth_pair, num_pairs};
use ratatui::buffer::Buffer;
use ratatui::layout::Alignment;
use ratatui::widgets::{BorderType, Tabs, Widget};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Line,
    widgets::Block,
};
use ratatui::{text::Span, widgets::Paragraph};

mod tables;
pub use tables::*;

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
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center);
        let inner = block.inner(area);
        block.render(area, buf);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(1),
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

pub fn help_text_widget() -> Paragraph<'static> {
    Paragraph::new(
        Line::from(vec![
            "[j/k]".fg(Color::LightGreen).bold(),
            Span::from(" navigate, "),
            "◄ ► to change tab".fg(Color::LightGreen).bold(),
            Span::from(" time, "),
            "[s]".fg(Color::LightGreen).bold(),
            Span::from(" stats, "),
            "[d]".fg(Color::LightGreen).bold(),
            Span::from(" distributions, "),
            "[f]".fg(Color::LightGreen).bold(),
            Span::from(" toggle filters,"),
            "[c]".fg(Color::LightGreen).bold(),
            Span::from(" toggle rolling, "),
            "[m]".fg(Color::LightGreen).bold(),
            Span::from(" toggle means"),
        ])
        .centered(),
    )
}
