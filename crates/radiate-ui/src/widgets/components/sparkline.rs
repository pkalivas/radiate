use crate::state::AppState;
use radiate_engines::Chromosome;
use ratatui::{
    style::{Color, Style},
    widgets::{Block, RenderDirection, Sparkline, SparklineBar, StatefulWidget, Widget},
};

pub struct SpeciesSparklineComponent<C> {
    _phantom: std::marker::PhantomData<C>,
}

impl<C> SpeciesSparklineComponent<C> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<C: Chromosome> StatefulWidget for SpeciesSparklineComponent<C> {
    type State = AppState<C>;

    fn render(
        self,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
        state: &mut Self::State,
    ) {
        let items = match state.evo.get_species() {
            Some(species) => species,
            None => return,
        };

        let bars = items
            .iter()
            .map(|species| {
                let color = if let Some(selected_id) = state.tables.species.selected_value {
                    if selected_id == species.id {
                        crate::styles::SELECTED_GREEN
                    } else {
                        Color::DarkGray
                    }
                } else {
                    Color::DarkGray
                };

                SparklineBar::from(species.size as u64).style(Style::default().fg(color))
            })
            .collect::<Vec<_>>();

        let sparkline = Sparkline::default()
            .block(Block::bordered())
            .data(bars)
            .direction(RenderDirection::LeftToRight)
            .style(Style::default().fg(Color::LightGreen))
            .max(items.iter().map(|s| s.size).max().unwrap_or(1) as u64);
        sparkline.render(area, buf);
    }
}
