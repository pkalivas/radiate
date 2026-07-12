use crate::widgets::Panel;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::Widget,
};

pub struct HelpPanelWidget;

impl Widget for HelpPanelWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let help_text = Text::from(vec![
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
            Line::from("  j / ↓       Move selection down"),
            Line::from("  k / ↑       Move selection up"),
            Line::from("  d / PgDn    Page down"),
            Line::from("  u / PgUp    Page up"),
            Line::from("  g / Home    Jump to top"),
            Line::from("  G / End     Jump to bottom"),
            Line::from("  h / ←       Previous tab"),
            Line::from("  l / →       Next tab"),
            Line::from(""),
            Line::from("Charts / Objective pairs"),
            Line::from("  [ / ]       Prev / next objective-pair page"),
            Line::from("  + / -       Expand / shrink objective pairs"),
            Line::from("  Enter       Toggle metric chart modal"),
            Line::from("  0-9         Select a specific objective index."),
            Line::from(""),
            Line::from("Filters"),
            Line::from("  /           Move focus to search bar"),
            Line::from("  Enter       Select search and move focus back"),
            Line::from("  Esc         Clear all search filters"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Press Esc to close",
                Style::default().add_modifier(Modifier::DIM),
            )]),
        ]);

        Panel::new(help_text).titled(" Help ").render(area, buf);
    }
}
