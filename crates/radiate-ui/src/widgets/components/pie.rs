use crate::{
    state::AppState, styles::COLOR_WHEEL_400, widgets::AppWidget,
    widgets::panels::tables::tagged_metrics,
};
use radiate_engines::{Chromosome, metric_names, stats::TagType};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Widget},
};
use tui_piechart::{PieChart, PieSlice};

pub struct SpeciesPieChartComponent;

impl SpeciesPieChartComponent {
    pub fn new() -> Self {
        Self
    }
}

impl<C: Chromosome> AppWidget<C> for SpeciesPieChartComponent {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppState<C>) {
        let Some(species) = &state.evo.get_species() else {
            let block = Block::bordered().title(Line::from(" No Data ").centered());
            block.render(area, buf);
            return;
        };

        let obj_idx = state.evo.pareto.objective_index;
        let selected_id = state.tables.species.selected_value.as_ref();
        let mut total = 0.0;
        let mut selected_value = 0.0;

        let slices = species
            .iter()
            .enumerate()
            .filter_map(|(index, species)| {
                species.adj_score().as_ref().map(|score| {
                    let color = selected_chart_color(index, selected_id, &species.id);
                    let name = radiate_utils::intern!(format!("{}", species.id.as_ref()));

                    let value = score[obj_idx] as f64;
                    total += value;
                    if Some(&species.id) == selected_id {
                        selected_value = value;
                    }

                    PieSlice::new(name, score[obj_idx] as f64, color)
                })
            })
            .collect::<Vec<_>>();

        let species_contrib = if total == 0.0 {
            " No Data ".to_string()
        } else if selected_value < total {
            format!(" {:.2}% ", (selected_value / total * 100.0))
        } else {
            " 100% ".to_string()
        };

        PieChart::new(slices)
            .show_legend(false)
            .show_percentages(true)
            .block(
                Block::bordered()
                    .title(
                        Line::from(Span::styled(
                            if state.evo.pareto.objective.dims() > 1 {
                                format!(
                                    " Adj Scores {} (Obj {}) ",
                                    selected_id.map(|id| *(*id).as_ref()).unwrap_or(0),
                                    obj_idx
                                )
                            } else {
                                format!(
                                    " Adj Scores {} ",
                                    selected_id.map(|id| *(*id).as_ref()).unwrap_or(0)
                                )
                            },
                            Style::default().fg(Color::White).bold(),
                        ))
                        .centered(),
                    )
                    .title_bottom(
                        Line::from(Span::styled(
                            format!(" {} ", species_contrib),
                            Style::default().fg(Color::White).bold(),
                        ))
                        .centered(),
                    ),
            )
            .legend_layout(tui_piechart::LegendLayout::Horizontal)
            .high_resolution(true)
            .render(area, buf);
    }
}

pub struct TimePieChartComponent;

impl TimePieChartComponent {
    pub fn new() -> Self {
        Self
    }
}

impl<C: Chromosome> AppWidget<C> for TimePieChartComponent {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppState<C>) {
        let items = tagged_metrics(&state.evo.metrics, state, TagType::Time)
            .iter()
            .filter(|met| met.name() != &metric_names::TIME)
            .copied()
            .collect::<Vec<_>>();

        let selected_name = state
            .tables
            .time
            .selected_value
            .as_deref()
            .unwrap_or("None");

        let mut total = 0.0;
        let mut selected_value = 0.0;

        let slices = items
            .iter()
            .enumerate()
            .map(|(index, metric)| {
                let label = metric.name();
                let color = selected_chart_color(index, Some(selected_name), label);
                let value = metric
                    .times()
                    .map(|t| t.sum())
                    .map(|d| d.as_millis() as f64)
                    .unwrap_or(0.0);

                total += value;
                if *label == selected_name {
                    selected_value = value;
                }

                PieSlice::new(label, value, color)
            })
            .collect::<Vec<_>>();

        let metric_contrib = if total == 0.0 {
            " No Data ".to_string()
        } else if selected_value < total {
            format!(" {:.2}% ", (selected_value / total * 100.0))
        } else {
            " 100% ".to_string()
        };

        PieChart::new(slices)
            .show_legend(false)
            .show_percentages(true)
            .block(
                Block::bordered()
                    .title(
                        Line::from(Span::styled(
                            format!(" {} ", selected_name),
                            Style::default().fg(Color::White).bold(),
                        ))
                        .centered(),
                    )
                    .title_bottom(
                        Line::from(Span::styled(
                            format!(" {} ", metric_contrib),
                            Style::default().fg(Color::White).bold(),
                        ))
                        .centered(),
                    ),
            )
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
