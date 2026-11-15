use crate::{
    state::AppState,
    widgets::{AppWidget, FnWidget, Panel},
};
use color_eyre::owo_colors::Style;
use radiate_engines::Chromosome;
use ratatui::{
    prelude::{Buffer, Rect},
    style::Color,
    text::Span,
    widgets::Widget,
};
use std::fmt::Debug;

pub struct DebugPanel;

impl DebugPanel {
    pub fn new() -> Self {
        DebugPanel
    }
}

impl<C: Chromosome + Debug> AppWidget<C> for DebugPanel {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppState<C>) {
        let selected_phenotype = state.get_selected_phenotype();

        let displays = if let Some(phenotype) = selected_phenotype {
            phenotype
                .genotype()
                .iter()
                .map(|gene| format!("{:?}", gene))
                .collect::<Vec<String>>()
        } else {
            vec!["No phenotype selected".to_string()]
        };

        let paragraph = displays.join("\n");

        Panel::new(FnWidget::new(|area, buf| {
            for (i, line) in paragraph.lines().enumerate() {
                buf.set_string(area.x, area.y + i as u16, line, Color::White);
            }
        }))
        // .titled(Span::styled(
        //     format!(" hi "),
        //     Style::default().fg(Color::White).bold(),
        // ))
        .render(area, buf);

        // let mut y = area.y;
        // for display in displays {
        //     buf.set_string(area.x, y, display, Color::White);
        //     y += 1;
        // }
    }
}
