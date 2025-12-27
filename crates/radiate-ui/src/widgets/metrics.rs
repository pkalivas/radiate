use crate::widgets::{FnWidget, Panel};
use crate::{
    state::{AppState, MetricsTab},
    styles,
    widgets::{FilterWidget, StatsTableWidget, TimeTableWidget},
};
use radiate_engines::Chromosome;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{StatefulWidget, Tabs, Widget},
};

pub struct MetricsWidget<C: Chromosome> {
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Chromosome> MetricsWidget<C> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<C: Chromosome> StatefulWidget for MetricsWidget<C> {
    type State = AppState<C>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let metric_table_area = if state.display.show_tag_filters {
            let [left, right] =
                Layout::horizontal([Constraint::Length(20), Constraint::Fill(1)]).areas(area);
            Panel::new(FilterWidget::new(&mut state.filter_state))
                .titled(" Filters ")
                .render(left, buf);
            right
        } else {
            area
        };

        Panel::new(FnWidget::new(|area, buf| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Fill(1)])
                .split(area);

            let titles = ["Stats", "Time"]
                .into_iter()
                .map(|t| Span::styled(format!(" {t} "), Style::default().fg(Color::White)));

            let index = match state.metrics_tab {
                MetricsTab::Stats => 0,
                MetricsTab::Time => 1,
            };

            Tabs::new(titles)
                .select(index)
                .padding(" ", " ")
                .divider(" ")
                .highlight_style(styles::selected_item_style())
                .bold()
                .render(chunks[0], buf);

            match state.metrics_tab {
                MetricsTab::Time => TimeTableWidget::new(state).render(chunks[1], buf),
                MetricsTab::Stats => StatsTableWidget::new(state).render(chunks[1], buf),
            }
        }))
        .titled(" Metrics ")
        .titled_bottom(help_text_minimal())
        .render(metric_table_area, buf);
    }
}

fn help_text_minimal<'a>() -> Line<'a> {
    Line::from(vec![
        " [j/k]".fg(Color::LightGreen).bold(),
        Span::from(" navigate, "),
        "[◄ ►/h/l]".fg(Color::LightGreen).bold(),
        Span::from(" change tab, "),
        "[f]".fg(Color::LightGreen).bold(),
        Span::from(" toggle filters, "),
        "[?/H]".fg(Color::LightGreen).bold(),
        Span::from(" help "),
    ])
    .centered()
}
