use crate::state::{AppState, AppTableState, Pane};
use crate::widgets::AppWidget;
use radiate_engines::stats::TagType;
use radiate_engines::{Chromosome, MetricSet, Species, metric_names};
use radiate_engines::{Metric, stats::fmt_duration};
use ratatui::buffer::Buffer;
use ratatui::text::{Line, Span};
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Cell, Row, Table},
};
use std::iter::{once, repeat_n};

pub const STAT_HEADER_CELLS: [&str; 6] = ["Metric", "Last", "Min", "Max", "μ (mean)", "Count"];
pub const TIME_HEADER_CELLS: [&str; 5] = ["Metric", "Min", "Max", "μ (mean)", "Total"];
pub const SPECIES_HEADER_CELLS: [&str; 6] =
    ["ID", "Age", "Size", "Gen. Stag", "Raw Score", "Adj. Score"];
pub const DIST_HEADER_CELLS: [&str; 7] = [
    "Metric",
    "Min",
    "Max",
    "μ (mean)",
    "Std Dev",
    "Var",
    "Count",
];

// --- Metric table ---

pub enum MetricTableKind {
    Time,
    Stats,
    Distribution,
}

impl MetricTableKind {
    fn tag(&self) -> TagType {
        match self {
            Self::Time => TagType::Time,
            Self::Stats => TagType::Statistic,
            Self::Distribution => TagType::Distribution,
        }
    }

    fn headers(&self) -> &'static [&'static str] {
        match self {
            Self::Time => &TIME_HEADER_CELLS,
            Self::Stats => &STAT_HEADER_CELLS,
            Self::Distribution => &DIST_HEADER_CELLS,
        }
    }

    fn widths(&self) -> Vec<Constraint> {
        match self {
            Self::Time => vec![Constraint::Fill(1); 5],
            Self::Stats => once(Constraint::Length(25))
                .chain(repeat_n(Constraint::Fill(1), 5))
                .collect(),
            Self::Distribution => once(Constraint::Length(25))
                .chain(repeat_n(Constraint::Fill(1), 6))
                .collect(),
        }
    }

    fn filter_item(&self, name: &str) -> bool {
        match self {
            Self::Time => name != metric_names::TIME,
            _ => true,
        }
    }

    fn build_rows<'a>(&self, items: impl Iterator<Item = &'a Metric>) -> Vec<Row<'a>> {
        match self {
            Self::Time => metric_to_time_rows(items).collect(),
            Self::Stats => metrics_into_stat_rows(items).collect(),
            Self::Distribution => metrics_into_dist_rows(items).collect(),
        }
    }
}

pub struct MetricTableWidget {
    kind: MetricTableKind,
}

impl MetricTableWidget {
    pub fn time() -> Self {
        Self {
            kind: MetricTableKind::Time,
        }
    }

    pub fn stats() -> Self {
        Self {
            kind: MetricTableKind::Stats,
        }
    }

    pub fn distribution() -> Self {
        Self {
            kind: MetricTableKind::Distribution,
        }
    }
}

impl<C: Chromosome> AppWidget<C> for MetricTableWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppState<C>) {
        let items: Vec<_> = tagged_metrics(&state.evo.metrics, state, self.kind.tag())
            .into_iter()
            .filter(|m| self.kind.filter_item(m.name()))
            .collect();

        match self.kind {
            MetricTableKind::Time => state.tables.time.update_rows(&items, |m| m.name().clone()),
            MetricTableKind::Stats => state.tables.stats.update_rows(&items, |m| m.name().clone()),
            MetricTableKind::Distribution => {
                state.tables.dist.update_rows(&items, |m| m.name().clone())
            }
        }

        let focused = state.nav.is_pane_focused(Pane::List);
        let border_style = crate::styles::panel_block(focused);

        let rows = self.kind.build_rows(items.iter().copied());

        let table = Table::default()
            .block(border_style)
            .header(header_row(self.kind.headers()))
            .rows(striped_rows(rows))
            .row_highlight_style(crate::styles::selected_item_style())
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
            .widths(self.kind.widths());

        match self.kind {
            MetricTableKind::Time => {
                render_scrollable_table(buf, area, table, &mut state.tables.time)
            }
            MetricTableKind::Stats => {
                render_scrollable_table(buf, area, table, &mut state.tables.stats)
            }
            MetricTableKind::Distribution => {
                render_scrollable_table(buf, area, table, &mut state.tables.dist)
            }
        }
    }
}

pub struct SpeciesTableWidget;

impl SpeciesTableWidget {
    pub fn new() -> Self {
        Self
    }
}

impl<C: Chromosome> AppWidget<C> for SpeciesTableWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppState<C>) {
        let items = match state.evo.get_species() {
            Some(species) => species,
            None => return,
        };

        state.tables.species.update_rows(items, |s| s.id);

        let obj_index = state.evo.pareto.objective_index;
        let generation = state.evo.index;
        let border_style = crate::styles::panel_block(state.nav.is_pane_focused(Pane::List));
        let rows = species_into_rows(obj_index, generation, items);

        let table = Table::default()
            .block(border_style)
            .header(header_row(&SPECIES_HEADER_CELLS))
            .rows(striped_rows(rows))
            .row_highlight_style(crate::styles::selected_item_style())
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
            .widths(
                (0..SPECIES_HEADER_CELLS.len())
                    .map(|_| Constraint::Fill(1))
                    .collect::<Vec<_>>(),
            );

        render_scrollable_table(buf, area, table, &mut state.tables.species);
    }
}

// --- Shared helpers ---

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
        let scrollbar_state = state
            .scroll_bar
            .get_or_insert_with(|| ScrollbarState::new(state.row_count));

        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .track_style(Style::default().fg(Color::DarkGray))
            .thumb_style(Style::default().fg(Color::LightGreen));

        scrollbar.render(scroll, buf, scrollbar_state);
    }
}

pub fn tagged_metrics<'a, C: Chromosome>(
    metrics: &'a MetricSet,
    state: &AppState<C>,
    tag: TagType,
) -> Vec<&'a Metric> {
    let mut items = metrics
        .iter_tagged(tag)
        .filter(|m| state.metric_matches_search(m))
        .collect::<Vec<_>>();
    items.sort_unstable_by(|a, b| a.name().cmp(&b.name()));
    items
}

// --- Row builders ---

fn metric_to_time_rows<'a>(
    metrics: impl Iterator<Item = &'a Metric>,
) -> impl Iterator<Item = Row<'a>> {
    metrics.filter_map(|m| {
        m.times().map(|time| {
            Row::new(vec![
                Cell::from(m.name().to_string()),
                Cell::from(fmt_duration(time.min())),
                Cell::from(fmt_duration(time.max())),
                Cell::from(fmt_duration(time.mean())),
                Cell::from(fmt_duration(time.sum())),
            ])
        })
    })
}

fn metrics_into_stat_rows<'a>(
    metrics: impl Iterator<Item = &'a Metric>,
) -> impl Iterator<Item = Row<'a>> {
    metrics.filter_map(|m| {
        m.stats().map(|stat| {
            let last = stat.last();
            let mean = stat.mean();
            let color = crate::styles::trend_color(last, mean);
            let symbol = crate::styles::trend_symbol(last, mean);
            Row::new(vec![
                Cell::from(Line::from(m.name().to_string())),
                Cell::from(Span::styled(
                    format!("{} {:.2}", symbol, last),
                    Style::default().fg(color),
                )),
                Cell::from(format!("{:.2}", stat.min())),
                Cell::from(format!("{:.2}", stat.max())),
                Cell::from(format!("{:.2}", mean)),
                Cell::from(format!("{}", stat.count())),
            ])
        })
    })
}

fn metrics_into_dist_rows<'a>(
    metrics: impl Iterator<Item = &'a Metric>,
) -> impl Iterator<Item = Row<'a>> {
    metrics.filter_map(|m| {
        m.distributions().map(|stat| {
            Row::new(vec![
                Cell::from(Line::from(m.name().to_string())),
                Cell::from(format!("{:.2}", stat.min())),
                Cell::from(format!("{:.2}", stat.max())),
                Cell::from(format!("{:.2}", stat.mean())),
                Cell::from(format!("{:.2}", stat.stddev())),
                Cell::from(format!("{:.2}", stat.var())),
                Cell::from(format!("{}", stat.count())),
            ])
        })
    })
}

fn species_into_rows<'a, C: Chromosome>(
    obj_index: usize,
    generation: usize,
    species: &[Species<C>],
) -> impl Iterator<Item = Row<'a>> {
    species.iter().map(move |s| {
        Row::new(vec![
            Cell::from(format!("{}", s.id.as_ref())),
            Cell::from(format!("{}", s.age(generation))),
            Cell::from(format!("{}", s.size)),
            Cell::from(format!("{}", s.stagnation())),
            Cell::from(format!(
                "{:.4}",
                s.tracker
                    .best
                    .as_ref()
                    .map(|vals| vals[obj_index])
                    .unwrap_or_default()
            )),
            Cell::from(format!(
                "{:.4}",
                s.adjusted_score
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
        .map(|(i, row)| row.style(crate::styles::alternating_row_style(i)))
}

fn header_row<'a>(cols: &'a [&str]) -> Row<'a> {
    Row::new(cols.iter().copied().map(Cell::from))
        .height(1)
        .style(Style::default().bold().underlined().fg(Color::White))
}
