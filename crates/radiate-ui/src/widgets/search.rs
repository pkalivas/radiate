use crate::{
    state::AppState,
    widgets::{FnWidget, Panel},
};
use radiate_engines::Chromosome;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Borders, Paragraph, Widget},
};

pub struct SearchBarWidget<'a, C: Chromosome> {
    pub state: &'a AppState<C>,
}

impl<'a, C: Chromosome> SearchBarWidget<'a, C> {
    pub fn new(state: &'a AppState<C>) -> Self {
        Self { state }
    }
}

impl<'a, C: Chromosome> Widget for SearchBarWidget<'a, C> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = if self.state.search_state.active {
            " Search (active) "
        } else {
            " Search (/) "
        };

        let style = if self.state.search_state.active {
            Style::default()
        } else {
            Style::default()
        };

        let border_style = self.state.get_panel_block(crate::state::PanelId::Search);

        Panel::new(FnWidget::new(|area, buf| {
            Paragraph::new(self.state.search_state.query.as_str())
                .block(border_style.title(title).style(style).borders(Borders::ALL))
                .style(style)
                .render(area, buf);
        }))
        .render(area, buf);
    }
}
