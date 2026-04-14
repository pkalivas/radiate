use crate::state::{AppState, AppTableState, PanelId};
use crate::styles::{self, COLOR_WHEEL_400};
use crate::widgets::MetricDetailPanelWidget;
use crate::widgets::components::PieChartWidget;
use radiate_engines::stats::TagType;
use radiate_engines::{Chromosome, MetricSet, SpeciesSnapshot, metric_names};
use radiate_engines::{Metric, stats::fmt_duration};
use ratatui::buffer::Buffer;
use ratatui::text::Line;
use ratatui::widgets::{
    RenderDirection, Scrollbar, ScrollbarOrientation, ScrollbarState, Sparkline, SparklineBar,
};
use ratatui::widgets::{StatefulWidget, Widget};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Cell, Row, Table},
};
use std::iter::{once, repeat};
use tui_piechart::{PieChart, PieSlice};

pub const STAT_HEADER_CELLS: [&str; 8] = [
    "Metric",
    "Min",
    "Max",
    "μ (mean)",
    "Sum",
    "StdDev",
    "Var",
    "Count",
];

pub const TIME_HEADER_CELLS: [&str; 5] = ["Metric", "Min", "Max", "μ (mean)", "Total"];

pub const SPECIES_HEADER_CELLS: [&str; 6] = ["ID", "Gen", "Pop", "Stag", "Best", "Score"];

pub struct TimeTableWidget<C: Chromosome> {
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Chromosome> TimeTableWidget<C> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<C: Chromosome> StatefulWidget for TimeTableWidget<C> {
    type State = AppState<C>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // let [middle_left, middle_right] =
        //     Layout::horizontal([Constraint::Fill(1), Constraint::Percentage(20)]).areas(area);

        // MetricDetailPanelWidget::new().render(middle_right, buf, state);

        let items = tagged_metrics(&state.metrics, state, TagType::Time)
            .iter()
            .filter(|met| met.0 != metric_names::TIME)
            .map(|m| *m)
            .collect::<Vec<_>>();
        state.time_table.update_rows(&items, |(name, _)| name);
        let border_style = state.get_panel_block(PanelId::TimeTable);

        // let [left, right] = Layout::horizontal([Constraint::Percentage(30), Constraint::Fill(1)])
        //     .areas(middle_left);

        // PieChartWidget::new(
        //     &items,
        //     |(name, _)| *name,
        //     |(_, metric)| {
        //         metric
        //             .times()
        //             .and_then(|t| t.sum())
        //             .map(|d| d.as_millis() as f64)
        //             .unwrap_or(0.0)
        //     },
        //     |(name, _)| *name,
        // )
        // .selected(state.time_table.selected_value)
        // .render(left, buf);

        let table = Table::default()
            .block(border_style)
            .header(header_row(&TIME_HEADER_CELLS))
            .rows(striped_rows(metric_to_time_rows(items.into_iter())))
            .row_highlight_style(styles::selected_item_style())
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
            .widths(&[
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ]);

        render_scrollable_table(buf, area, table, &mut state.time_table);
    }
}

pub struct StatsTableWidget<C: Chromosome> {
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Chromosome> StatsTableWidget<C> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<C: Chromosome> StatefulWidget for StatsTableWidget<C> {
    type State = AppState<C>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // let [middle_left, middle_right] =
        //     Layout::horizontal([Constraint::Fill(1), Constraint::Percentage(20)]).areas(area);
        // MetricDetailPanelWidget::new().render(middle_right, buf, state);

        let items = tagged_metrics(&state.metrics, state, TagType::Statistic);

        state.stats_table.update_rows(&items, |(name, _)| name);
        let border_style = state.get_panel_block(crate::state::PanelId::StatsTable);

        let table = Table::default()
            .block(border_style)
            .header(header_row(&STAT_HEADER_CELLS))
            .rows(striped_rows(metrics_into_stat_rows(items.into_iter())))
            .row_highlight_style(styles::selected_item_style())
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
            .widths(once(Constraint::Length(22)).chain(repeat(Constraint::Fill(1)).take(7)));

        render_scrollable_table(buf, area, table, &mut state.stats_table);
    }
}

pub struct DistributionTableWidget<C: Chromosome> {
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Chromosome> DistributionTableWidget<C> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<C: Chromosome> StatefulWidget for DistributionTableWidget<C> {
    type State = AppState<C>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let [middle_left, middle_right] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Percentage(20)]).areas(area);
        MetricDetailPanelWidget::new().render(middle_right, buf, state);

        let items = tagged_metrics(&state.metrics, state, TagType::Distribution);

        state.dist_table.update_rows(&items, |(name, _)| name);
        let border_style = state.get_panel_block(crate::state::PanelId::DistTable);

        let table = Table::default()
            .block(border_style)
            .header(header_row(&STAT_HEADER_CELLS))
            .rows(striped_rows(metrics_into_dist_rows(items.into_iter())))
            .row_highlight_style(styles::selected_item_style())
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
            .widths(once(Constraint::Length(22)).chain(repeat(Constraint::Fill(1)).take(7)));

        render_scrollable_table(buf, middle_left, table, &mut state.dist_table);
    }
}

pub struct SpeciesTableWidget<C: Chromosome> {
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Chromosome> SpeciesTableWidget<C> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<C: Chromosome> StatefulWidget for SpeciesTableWidget<C> {
    type State = AppState<C>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let items = match &state.species {
            Some(species) => species,
            None => return,
        };

        state.species_table.update_rows(&items, |s| s.id);

        let obj_index = state.objective_state.objective_index;
        let border_style = state.get_panel_block(PanelId::SpeciesTable);
        let rows = species_into_rows(obj_index, items.iter());

        let bars = items
            .iter()
            .map(|species| {
                let color = if let Some(selected_id) = state.species_table.selected_value {
                    if selected_id == species.id {
                        crate::styles::SELECTED_GREEN
                    } else {
                        Color::DarkGray
                    }
                } else {
                    Color::DarkGray
                };

                SparklineBar::from(species.population_size as u64).style(Style::default().fg(color))
            })
            .collect::<Vec<_>>();

        let sparkline = Sparkline::default()
            .block(Block::bordered())
            .data(bars)
            .direction(RenderDirection::LeftToRight)
            .style(Style::default().fg(Color::LightGreen))
            .max(items.iter().map(|s| s.population_size).max().unwrap_or(1) as u64);

        // let slices = items
        //     .iter()
        //     .enumerate()
        //     .filter_map(|(index, species)| {
        //         species.best_score.as_ref().map(|score| {
        //             let color = selected_chart_color(
        //                 index,
        //                 state.species_table.selected_value.as_ref(),
        //                 &species.id,
        //             );

        //             let name = radiate_utils::intern!(format!("{}", species.id.0));
        //             PieSlice::new(name, score[0] as f64, color)
        //         })
        //     })
        //     .collect::<Vec<_>>();

        // let piechart = PieChart::new(slices)
        //     .show_legend(false)
        //     .show_percentages(true)
        //     .block(Block::bordered())
        //     .legend_layout(tui_piechart::LegendLayout::Horizontal)
        //     .high_resolution(true);

        // let [left, middle, right] = Layout::horizontal([
        //     Constraint::Fill(1),
        //     Constraint::Percentage(25),
        //     Constraint::Percentage(25),
        // ])
        // .areas(area);

        // sparkline.render(middle, buf);
        // piechart.render(right, buf);

        let table = Table::default()
            .block(border_style)
            .header(header_row(&SPECIES_HEADER_CELLS))
            .rows(striped_rows(rows))
            .row_highlight_style(styles::selected_item_style())
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
            .widths(once(Constraint::Length(22)).chain(repeat(Constraint::Fill(1)).take(7)));

        render_scrollable_table(buf, area, table, &mut state.species_table);
    }
}

fn render_scrollable_table<T>(
    buf: &mut Buffer,
    area: Rect,
    table: Table,
    state: &mut AppTableState<T>,
) {
    let [tbl, scroll] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

    StatefulWidget::render(&table, tbl, buf, &mut state.state);

    if state.row_count > tbl.height as usize {
        let mut scrollbar_state = state
            .scroll_bar
            .get_or_insert_with(|| ScrollbarState::new(state.row_count));

        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .track_style(Style::default().fg(Color::DarkGray))
            .thumb_style(Style::default().fg(Color::LightGreen));

        scrollbar.render(scroll, buf, &mut scrollbar_state);
    }
}

fn selected_chart_color<K: PartialEq>(index: usize, selected: Option<&K>, current: &K) -> Color {
    match selected {
        Some(sel) if sel == current => COLOR_WHEEL_400[index % COLOR_WHEEL_400.len()],
        _ => Color::DarkGray,
    }
}

pub fn tagged_metrics<'a, C: Chromosome>(
    metrics: &'a MetricSet,
    state: &AppState<C>,
    tag: TagType,
) -> Vec<(&'static str, &'a Metric)> {
    let mut items = metrics
        .iter_tagged(tag)
        .filter(|(_, m)| state.metric_has_tags(m))
        .filter(|(_, m)| state.metric_matches_search(m))
        .collect::<Vec<_>>();
    items.sort_unstable_by(|a, b| a.0.cmp(b.0));
    items
}

/// --- Row Builders ---
fn metric_to_time_rows<'a>(
    metrics: impl Iterator<Item = (&'static str, &'a Metric)>,
) -> impl Iterator<Item = Row<'a>> {
    metrics.filter_map(|(name, m)| {
        if let Some(time) = m.times() {
            let mean = fmt_duration(time.mean().unwrap_or_default());
            let min = fmt_duration(time.min().unwrap_or_default());
            let max = fmt_duration(time.max().unwrap_or_default());
            let total = fmt_duration(time.sum().unwrap_or_default());

            Some(Row::new(vec![
                Cell::from(name.to_string()),
                Cell::from(min),
                Cell::from(max),
                Cell::from(mean),
                Cell::from(total),
            ]))
        } else {
            None
        }
    })
}

fn metrics_into_stat_rows<'a>(
    metrics: impl Iterator<Item = (&'static str, &'a Metric)>,
) -> impl Iterator<Item = Row<'a>> {
    metrics.filter_map(|(name, m)| {
        if let Some(stat) = m.stats() {
            Some(Row::new(vec![
                Cell::from(Line::from(name.to_string())),
                Cell::from(format!("{:.2}", stat.min().unwrap_or_default())),
                Cell::from(format!("{:.2}", stat.max().unwrap_or_default())),
                Cell::from(format!("{:.2}", stat.mean().unwrap_or_default())),
                Cell::from(format!("{:.2}", stat.sum().unwrap_or_default())),
                Cell::from(format!("{:.2}", stat.stddev().unwrap_or(0.0))),
                Cell::from(format!("{:.2}", stat.var().unwrap_or(0.0))),
                Cell::from(format!("{}", stat.count())),
            ]))
        } else {
            None
        }
    })
}

fn metrics_into_dist_rows<'a>(
    metrics: impl Iterator<Item = (&'static str, &'a Metric)>,
) -> impl Iterator<Item = Row<'a>> {
    metrics.filter_map(|(name, m)| {
        if let Some(stat) = m.distributions() {
            Some(Row::new(vec![
                Cell::from(Line::from(name.to_string())),
                Cell::from(format!("{:.2}", stat.min().unwrap_or_default())),
                Cell::from(format!("{:.2}", stat.max().unwrap_or_default())),
                Cell::from(format!("{:.2}", stat.mean().unwrap_or_default())),
                Cell::from(format!("{:.2}", stat.sum().unwrap_or_default())),
                Cell::from(format!("{:.2}", stat.stddev().unwrap_or(0.0))),
                Cell::from(format!("{:.2}", stat.var().unwrap_or(0.0))),
                Cell::from(format!("{}", stat.count())),
            ]))
        } else {
            None
        }
    })
}

fn species_into_rows<'a>(
    obj_index: usize,
    species: impl Iterator<Item = &'a SpeciesSnapshot>,
) -> impl Iterator<Item = Row<'a>> {
    species.map(move |s| {
        Row::new(vec![
            Cell::from(format!("{}", s.id.0)),
            Cell::from(format!("{}", s.generation)),
            Cell::from(format!("{}", s.population_size)),
            Cell::from(format!("{}", s.stagnation)),
            Cell::from(format!(
                "{}",
                s.best_score
                    .as_ref()
                    .map(|vals| vals[obj_index])
                    .unwrap_or_default()
            )),
            Cell::from(format!(
                "{}",
                s.score
                    .as_ref()
                    .map(|vals| vals[obj_index])
                    .unwrap_or_default()
            )),
        ])
    })
}

fn striped_rows<'a>(rows: impl IntoIterator<Item = Row<'a>>) -> impl Iterator<Item = Row<'a>> {
    rows.into_iter()
        .enumerate()
        .map(|(i, row)| row.style(styles::alternating_row_style(i)))
}

fn header_row<'a>(cols: &'a [&str]) -> Row<'a> {
    Row::new(cols.iter().copied().map(Cell::from))
        .height(1)
        .style(Style::default().bold().underlined().fg(Color::White))
}

// let slices = items
//     .iter()
//     .enumerate()
//     .filter_map(|(index, (name, m))| {
//         m.times().map(|time| {
//             let total_ms = time.sum().map(|d| d.as_millis()).unwrap_or(0) as f64;

//             let color = if let Some(selected_name) = state.time_table.selected_value {
//                 if selected_name == *name {
//                     COLOR_WHEEL_400[index % COLOR_WHEEL_400.len()]
//                 } else {
//                     Color::DarkGray
//                 }
//             } else {
//                 Color::DarkGray
//             };

//             PieSlice::new(name, total_ms, color)
//         })
//     })
//     .collect::<Vec<_>>();

// let piechart = PieChart::new(slices)
//     .show_legend(false)
//     .show_percentages(true)
//     .block(Block::bordered())
//     .legend_layout(tui_piechart::LegendLayout::Horizontal)
//     .high_resolution(true);

// fn create_pie_chart<'a, T, K, F>(items: &[T], selected: Option<&K>, func: F) -> PieChart<'a>
// where
//     F: Fn(&T) -> K,
//     K: PartialEq + Debug,
// {
//     let slices = items
//         .iter()
//         .enumerate()
//         .filter_map(|(index, value)| {
//             let item_value = func(value);
//             let name = format!("{:?}", item_value);
//             let color = if let Some(selected) = selected {
//                 if item_value == *selected {
//                     COLOR_WHEEL_400[index % COLOR_WHEEL_400.len()]
//                 } else {
//                     Color::DarkGray
//                 }
//             } else {
//                 Color::DarkGray
//             };

//             PieSlice::new(name, 1.0, color)
//         })
//         .collect::<Vec<_>>();

//     PieChart::new(slices)
//         .show_legend(false)
//         .show_percentages(true)
//         .block(Block::bordered())
//         .legend_layout(tui_piechart::LegendLayout::Horizontal)
//         .high_resolution(true)
// }

// // let slices = items
// //     .iter()
// //     .enumerate()
// //     .filter_map(|(index, species)| {
// //         species.best_score.as_ref().map(|score| {
// //             let color = if let Some(selected_id) = self.state.species_table.selected_metric
// //             {
// //                 if selected_id == species.id {
// //                     COLOR_WHEEL_400[index % COLOR_WHEEL_400.len()]
// //                 } else {
// //                     Color::DarkGray
// //                 }
// //             } else {
// //                 Color::DarkGray
// //             };

// //             let name = radiate_utils::intern!(format!("{}", species.id.0));
// //             PieSlice::new(name, score[0] as f64, color)
// //         })
// //     })
// //     .collect::<Vec<_>>();

// // let piechart = PieChart::new(slices)
// //     .show_legend(false)
// //     .show_percentages(true)
// //     .block(Block::bordered())
// //     .legend_layout(tui_piechart::LegendLayout::Horizontal)
// //     .high_resolution(true);
