use radiate_engines::{Chromosome, stats::metric_tags};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, StatefulWidget, Widget},
};

use crate::state::AppState;

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
        let block = Block::bordered().title(Line::from(" Filter ").centered());
        let inner = block.inner(area);
        block.render(area, buf);

        let tags = self
            .state
            .filter_state
            .all_tags
            .iter()
            .filter(|tag| {
                tag.0 != metric_tags::STATISTIC
                    && tag.0 != metric_tags::DISTRIBUTION
                    && tag.0 != metric_tags::TIME
            })
            .enumerate()
            .map(|(i, tag)| {
                if self.state.filter_state.tag_view.contains(&i) {
                    if i == self.state.filter_state.selected_row {
                        return ListItem::new(Span::styled(
                            format!(">  [x] {}", tag.0),
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        ));
                    } else {
                        return ListItem::new(Line::from(vec![
                            Span::raw("  ["),
                            Span::styled(
                                format!("X"),
                                Style::default()
                                    .fg(Color::Green)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(format!("] {}", tag.0)),
                        ]));
                    }
                } else {
                    if i == self.state.filter_state.selected_row {
                        return ListItem::new(Span::styled(
                            format!(">  [ ] {}", tag.0),
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        ));
                    } else {
                        ListItem::new(Span::styled(
                            format!("  [ ] {}", tag.0),
                            Style::default().fg(Color::White),
                        ))
                    }
                }
            })
            .collect::<Vec<_>>();

        StatefulWidget::render(
            List::new(tags),
            inner,
            buf,
            &mut self.state.filter_state.tag_list_filter_state,
        );
    }
}
