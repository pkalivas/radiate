use crate::{state::AppState, styles::COLOR_WHEEL_400, widgets::panels::tables::tagged_metrics};
use radiate_engines::{Chromosome, metric_names, stats::TagType};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    text::Line,
    widgets::{Block, StatefulWidget, Widget},
};
use tui_piechart::{PieChart, PieSlice};

pub struct SpeciesPieChartComponent<C: Chromosome> {
    _marker: std::marker::PhantomData<C>,
}

impl<C: Chromosome> SpeciesPieChartComponent<C> {
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<C: Chromosome> StatefulWidget for SpeciesPieChartComponent<C> {
    type State = AppState<C>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let Some(species) = &state.evo.get_species() else {
            let block = Block::bordered().title(Line::from(" No Data ").centered());
            block.render(area, buf);
            return;
        };

        let obj_idx = state.evo.pareto.objective_index;
        let slices = species
            .iter()
            .enumerate()
            .filter_map(|(index, species)| {
                species.adj_score().as_ref().map(|score| {
                    let color = selected_chart_color(
                        index,
                        state.tables.species.selected_value.as_ref(),
                        &species.id,
                    );

                    let name = radiate_utils::intern!(format!("{}", species.id.0));
                    PieSlice::new(name, score[obj_idx] as f64, color)
                })
            })
            .collect::<Vec<_>>();

        PieChart::new(slices)
            .show_legend(false)
            .show_percentages(true)
            .block(Block::bordered())
            .legend_layout(tui_piechart::LegendLayout::Horizontal)
            .high_resolution(true)
            .render(area, buf);
    }
}

pub struct TimePieChartComponent<C> {
    _marker: std::marker::PhantomData<C>,
}

impl<C> TimePieChartComponent<C> {
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<C: Chromosome> StatefulWidget for TimePieChartComponent<C> {
    type State = AppState<C>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let items = tagged_metrics(&state.evo.metrics, state, TagType::Time)
            .iter()
            .filter(|met| met.0 != metric_names::TIME)
            .copied()
            .collect::<Vec<_>>();

        let slices = items
            .iter()
            .enumerate()
            .map(|(index, (label, metric))| {
                let color =
                    selected_chart_color(index, state.tables.time.selected_value.as_deref(), label);
                let value = metric
                    .times()
                    .map(|t| t.sum())
                    .map(|d| d.as_millis() as f64)
                    .unwrap_or(0.0);

                PieSlice::new(label, value, color)
            })
            .collect::<Vec<_>>();

        PieChart::new(slices)
            .show_legend(false)
            .show_percentages(true)
            .block(Block::bordered())
            .legend_layout(tui_piechart::LegendLayout::Horizontal)
            .high_resolution(true)
            .render(area, buf);
    }
}

fn selected_chart_color<K: PartialEq + ?Sized>(
    index: usize,
    selected: Option<&K>,
    current: &K,
) -> Color {
    match selected {
        Some(sel) if sel == current => COLOR_WHEEL_400[index % COLOR_WHEEL_400.len()],
        _ => Color::DarkGray,
    }
}
