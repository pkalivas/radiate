use crate::state::PanelId;
use crate::widgets::{DistributionTableWidget, FnWidget, MetricSearchWidget, Panel};
use crate::{
    state::{AppState, MetricsTab},
    styles,
    widgets::{FilterWidget, StatsTableWidget, TimeTableWidget},
};
use radiate_engines::Chromosome;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
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
            let block = state.get_panel_block(PanelId::Filters);
            Panel::new(FilterWidget::new(&mut state.filter_state))
                .titled(" Filters ")
                .bordered(block)
                .render(left, buf);
            right
        } else {
            area
        };

        let metrics_summary = state.metrics.summary();

        let line = Line::from(vec![
            Span::raw(" "),
            metrics_summary.metrics.to_string().into(),
            Span::raw(" | "),
            metrics_summary.updates.to_string().into(),
            Span::raw(" "),
        ]);

        Panel::new(FnWidget::new(|area, buf| {
            let [top, middle, bottom] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Percentage(80),
                Constraint::Fill(1),
            ])
            .areas(area);

            let titles = ["Stats", "Time", "Distribution"]
                .into_iter()
                .map(|t| Span::styled(format!(" {t} "), Style::default().fg(Color::White)));

            let index = match state.metrics_tab {
                MetricsTab::Stats => 0,
                MetricsTab::Time => 1,
                MetricsTab::Distribution => 2,
            };

            Tabs::new(titles)
                .select(index)
                .padding(" ", " ")
                .divider(" ")
                .highlight_style(styles::selected_item_style())
                .bold()
                .render(top, buf);

            match state.metrics_tab {
                MetricsTab::Time => TimeTableWidget::new(state).render(middle, buf),
                MetricsTab::Stats => StatsTableWidget::new(state).render(middle, buf),
                MetricsTab::Distribution => DistributionTableWidget::new(state).render(middle, buf),
            }

            MetricSearchWidget::new(state).render(bottom, buf);
        }))
        .titled(" Metrics ")
        .title_top_right(line)
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
