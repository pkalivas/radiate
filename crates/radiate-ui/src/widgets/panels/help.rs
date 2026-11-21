use crate::keys::{BINDINGS, Binding};
use crate::widgets::{FnWidget, Panel};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Paragraph, Widget},
};

/// The key map, drawn from [`BINDINGS`] in two columns.
pub struct HelpPanelWidget;

impl Widget for HelpPanelWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let groups = groups();
        let width = BINDINGS
            .iter()
            .filter(|b| !b.help.is_empty())
            .map(|b| b.label().chars().count())
            .max()
            .unwrap_or(0);

        // Split between whole sections so the taller column is as short as it
        // can be.
        let heights = groups.iter().map(|(_, g)| g.len() + 2).collect::<Vec<_>>();
        let split = (0..=groups.len())
            .min_by_key(|&i| {
                let left = heights[..i].iter().sum::<usize>();
                let right = heights[i..].iter().sum::<usize>();
                left.max(right)
            })
            .unwrap_or(0);

        let column = |groups: &[(&str, Vec<&Binding>)]| {
            let mut text = Vec::new();
            for (name, bindings) in groups {
                text.push(Line::from(Span::styled(
                    name.to_string(),
                    Style::default().add_modifier(Modifier::BOLD),
                )));
                for binding in bindings {
                    text.push(Line::from(format!(
                        "  {:<width$}  {}",
                        binding.label(),
                        binding.help
                    )));
                }
                text.push(Line::from(""));
            }
            Paragraph::new(Text::from(text))
        };

        let (first, second) = groups.split_at(split);
        let body = FnWidget::new(|area: Rect, buf: &mut Buffer| {
            let [left, right] = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)])
                .horizontal_margin(1)
                .spacing(2)
                .areas(area);
            column(first).render(left, buf);
            column(second).render(right, buf);
        });

        Panel::new(body)
            .render_inside_block(true)
            .titled(" Help ")
            .titled_bottom(Line::from(Span::styled(
                " Esc to close ",
                Style::default().add_modifier(Modifier::DIM),
            )))
            .render(area, buf);
    }
}

/// Bindings with help text, grouped by section in the order sections first
/// appear.
fn groups() -> Vec<(&'static str, Vec<&'static Binding>)> {
    let mut groups: Vec<(&str, Vec<&Binding>)> = Vec::new();
    for binding in BINDINGS.iter().filter(|b| !b.help.is_empty()) {
        match groups.iter_mut().find(|(name, _)| *name == binding.group) {
            Some((_, group)) => group.push(binding),
            None => groups.push((binding.group, vec![binding])),
        }
    }
    groups
}
