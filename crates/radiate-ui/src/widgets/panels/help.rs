use crate::widgets::Panel;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
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
        ]);

        Panel::new(help_text).titled(" Help ").render(area, buf);
    }
}

pub struct HelpTextMinimal;

impl Widget for HelpTextMinimal {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Panel::new(super::help::help_text_minimal()).render(area, buf);
    }
}

pub fn help_text_minimal<'a>() -> Line<'a> {
    Line::from(vec![
        " [j/k]".fg(Color::LightGreen).bold(),
        Span::from(" navigate, "),
        "[◄ ►/h/l]".fg(Color::LightGreen).bold(),
        Span::from(" tabs, "),
        // "[f]".fg(Color::LightGreen).bold(),
        // Span::from(" toggle filters, "),
        "[?/H]".fg(Color::LightGreen).bold(),
        Span::from(" help "),
    ])
    .centered()
}
