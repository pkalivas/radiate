use crate::state::{AppState, MetricsTab};
use crate::styles;
use radiate_engines::Chromosome;
mod pareto;
pub use pareto::{ParetoFrontTemp, ParetoFrontWidget, kth_pair, num_pairs};
use ratatui::buffer::Buffer;
use ratatui::layout::Alignment;
use ratatui::text::Span;
use ratatui::widgets::{BorderType, Tabs, Widget};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::Block,
};
mod chart;
pub use chart::ChartWidget;

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
            .title_bottom(help_text_widget())
            .title_top(" Metrics ")
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center);
        let inner = block.inner(area);
        block.render(area, buf);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Fill(1)])
            .split(inner);

        let titles = ["Stats", "Time", "Dist"]
            .into_iter()
            .map(|t| Span::styled(format!(" {t} "), Style::default().fg(Color::White)));

        let index = match self.state.metrics_tab {
            MetricsTab::Stats => 0,
            MetricsTab::Time => 1,
            MetricsTab::Distributions => 2,
        };

        Tabs::new(titles)
            .select(index)
            .padding(" ", " ")
            .divider(" ")
            .highlight_style(styles::selected_item_style())
            .bold()
            .render(chunks[0], buf);

        match self.state.metrics_tab {
            MetricsTab::Time => TimeTableWidget::new(&mut self.state).render(chunks[1], buf),
            MetricsTab::Stats => StatsTableWidget::new(&mut self.state).render(chunks[1], buf),
            MetricsTab::Distributions => {
                DistributionTableWidget::new(&mut self.state).render(chunks[1], buf)
            }
        }
    }
}

pub fn help_text_widget<'a>() -> Line<'a> {
    Line::from(vec![
        "[j/k]".fg(Color::LightGreen).bold(),
        Span::from(" navigate, "),
        "[◄ ►/h/l]".fg(Color::LightGreen).bold(),
        Span::from(" change tab, "),
        "[f]".fg(Color::LightGreen).bold(),
        Span::from(" toggle filters, "),
        "[c]".fg(Color::LightGreen).bold(),
        Span::from(" chart metric, "),
        "[m]".fg(Color::LightGreen).bold(),
        Span::from(" chart metric mean "),
    ])
    .centered()
}
