use radiate_engines::{Chromosome, stats::metric_tags};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Widget},
};

use crate::state::AppState;

pub struct FilterWidget<'a, C: Chromosome> {
    state: &'a AppState<C>,
}

impl<'a, C: Chromosome> FilterWidget<'a, C> {
    pub fn new(state: &'a AppState<C>) -> Self {
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
            .all_tags
            .iter()
            .filter(|tag| {
                tag.0 != metric_tags::STATISTIC
                    && tag.0 != metric_tags::DISTRIBUTION
                    && tag.0 != metric_tags::TIME
            })
            .enumerate()
            .map(|(i, tag)| {
                if self.state.tag_view.contains(&i) {
                    ListItem::new(Span::styled(
                        format!("[{}] {}", i, tag.0),
                        Style::default()
                            .fg(Color::LightGreen)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else {
                    ListItem::new(Span::styled(
                        format!("[{}] {}", i, tag.0),
                        Style::default().fg(Color::White),
                    ))
                }
            })
            .collect::<Vec<_>>();

        List::new(tags).render(inner, buf)
    }
}
