use crate::{state::AppFilterState, styles::SELECTED_GREEN};
use radiate_engines::stats::TagKind;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, StatefulWidget, Widget},
};

pub struct FilterWidget<'a> {
    state: &'a mut AppFilterState,
}

impl<'a> FilterWidget<'a> {
    pub fn new(state: &'a mut AppFilterState) -> Self {
        Self { state }
    }
}

impl<'a> Widget for FilterWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let tags = self
            .state
            .all_tags
            .iter()
            .filter(|tag| *(*tag) != TagKind::Statistic && *(*tag) != TagKind::Time)
            .enumerate()
            .map(|(i, tag)| {
                if self.state.tag_view.contains(&i) {
                    if i == self.state.selected_row {
                        return ListItem::new(Span::styled(
                            format!("> X {}", TagKind::as_str(tag)),
                            Style::default()
                                .fg(SELECTED_GREEN)
                                .add_modifier(Modifier::BOLD),
                        ));
                    } else {
                        let spans = vec![
                            Span::raw("  "),
                            Span::styled(
                                format!("X"),
                                Style::default()
                                    .fg(SELECTED_GREEN)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(format!(" {}", TagKind::as_str(tag))),
                        ];
                        return ListItem::new(Line::from(spans));
                    }
                } else {
                    if i == self.state.selected_row {
                        return ListItem::new(Span::styled(
                            format!("> - {}", TagKind::as_str(tag)),
                            Style::default()
                                .fg(SELECTED_GREEN)
                                .add_modifier(Modifier::BOLD),
                        ));
                    } else {
                        ListItem::new(Span::styled(
                            format!("  - {}", TagKind::as_str(tag)),
                            Style::default().fg(Color::White),
                        ))
                    }
                }
            })
            .collect::<Vec<_>>();

        StatefulWidget::render(
            List::new(tags),
            area,
            buf,
            &mut self.state.tag_list_filter_state,
        );
    }
}
