use crate::state::AppState;
use radiate_engines::Chromosome;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
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
        let title = if self.state.nav.search.active {
            " Search (active) "
        } else {
            " Search (/) "
        };

        let style = if self.state.nav.search.active {
            Style::default()
        } else {
            Style::default()
        };

        let border_style = crate::styles::panel_block(self.state.nav.is_search_focused());

        Paragraph::new(self.state.nav.search.query.as_str())
            .block(
                border_style
                    .title(title)
                    .title_bottom(help_text_minimal())
                    .style(style)
                    .borders(Borders::ALL),
            )
            .style(style)
            .render(area, buf);
    }
}

pub fn help_text_minimal<'a>() -> Line<'a> {
    Line::from(vec![
        " [j/k]".fg(Color::LightGreen).bold(),
        Span::from(" navigate, "),
        "[◄ ►/h/l]".fg(Color::LightGreen).bold(),
        Span::from(" tabs, "),
        "[?/H]".fg(Color::LightGreen).bold(),
        Span::from(" help "),
    ])
    .centered()
}
