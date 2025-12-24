use crate::{state::AppState, styles::SELECTED_GREEN};
use radiate_engines::{Chromosome, stats::TagKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, StatefulWidget, Widget},
};

pub struct FilterWidget<'a, C: Chromosome> {
    state: &'a mut AppState<C>,
}

impl<'a, C: Chromosome> FilterWidget<'a, C> {
    pub fn new(state: &'a mut AppState<C>) -> Self {
        Self { state }
    }
}

impl<'a, C: Chromosome> Widget for FilterWidget<'a, C> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let tags = self
            .state
            .filter_state
            .all_tags
            .iter()
            .filter(|tag| *(*tag) != TagKind::Statistic && *(*tag) != TagKind::Time)
            .enumerate()
            .map(|(i, tag)| {
                if self.state.filter_state.tag_view.contains(&i) {
                    if i == self.state.filter_state.selected_row {
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
                    if i == self.state.filter_state.selected_row {
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
            &mut self.state.filter_state.tag_list_filter_state,
        );
    }
}
