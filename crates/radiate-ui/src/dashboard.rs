use crate::state::{AppState, MetricsTab};
use color_eyre::Result;
use radiate_engines::{
    Metric, MetricScope,
    stats::{fmt_duration, metric_tags},
};
use radiate_engines::{MetricSet, Objective, Score, metric_names};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    text::Span,
    widgets::{
        List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, TableState,
    },
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize, palette::material},
    text::Line,
    widgets::{Block, Cell, Row, Table},
};
use std::{
    io,
    time::{Duration, Instant},
};
use tui_piechart::{PieChart, PieSlice};

pub const NORMAL_ROW_BG: Color = material::GRAY.c800;
pub const ALT_ROW_BG_COLOR: Color = material::GRAY.c900;
pub const TEXT_FG_COLOR: Color = material::GRAY.c300;

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
pub const ENGINE_HEADER_CELLS: [&str; 2] = ["Metric", "Val"];
pub const DISTRIBUTION_HEADER_CELLS: [&str; 11] = [
    "Metric", "Min", ".25p", ".50p", ".75p", "Max", "Count", "StdDev", "Var", "Skew", "Entr.",
];

const COLORS: [Color; 8] = [
    material::RED.c400,
    material::BLUE.c400,
    material::GREEN.c400,
    material::YELLOW.c400,
    material::PURPLE.c400,
    material::CYAN.c400,
    material::ORANGE.c400,
    material::TEAL.c400,
];

pub(crate) struct Dashboard {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    state: AppState,
}

impl Dashboard {
    pub fn new(render_interval: Duration) -> Self {
        Self {
            terminal: ratatui::init(),
            state: AppState {
                render_interval,
                ..Default::default()
            },
        }
    }

    pub fn set_metric_tab(&mut self, tab: MetricsTab) {
        self.state.metrics_tab = tab;
        self.render().unwrap_or_default();
    }

    pub fn move_selection_down(&mut self) {
        self.state.move_selection_down();
        self.render().unwrap_or_default();
    }
    pub fn move_selection_up(&mut self) {
        self.state.move_selection_up();
        self.render().unwrap_or_default();
    }

    pub fn toggle_tag_filter_display(&mut self) {
        self.state.toggle_display_tag_filters();
        self.render().unwrap_or_default();
    }

    pub fn toggle_metric_charts_display(&mut self) {
        self.state.toggle_display_metric_charts();
        self.render().unwrap_or_default();
    }

    pub fn toggle_metric_means_display(&mut self) {
        self.state.toggle_display_metric_means();
        self.render().unwrap_or_default();
    }

    pub fn set_tag_filter_by_index(&mut self, index: usize) {
        self.state.set_tag_filter_by_index(index);
        self.render().unwrap_or_default();
    }

    pub fn toggle_is_running(&mut self) {
        self.state.toggle_is_running();
        self.render().unwrap_or_default();
    }

    pub fn try_render(&mut self) -> Result<()> {
        let now = Instant::now();
        if let Some(last) = self.state.last_render() {
            if now.duration_since(last) < self.state.render_interval() {
                return Ok(());
            }
        }

        self.state.set_last_render(Some(now));
        self.render()
    }

    pub fn render(&mut self) -> Result<()> {
        self.terminal.draw(|f| {
            Self::ui(f, &mut self.state);
        })?;

        Ok(())
    }

    pub fn update(&mut self, metrics: MetricSet, score: Score, index: usize, objective: Objective) {
        let charts = self.state.charts_mut();

        charts
            .fitness_chart_mut()
            .update_last_value(index as f64, score.as_f32() as f64);

        if let Some(dist) = metrics
            .get(metric_names::SCORES)
            .and_then(|m| m.distribution())
        {
            charts
                .fitness_chart_mut()
                .update_mean_value(index as f64, dist.mean() as f64);
        }

        for metric in metrics.iter() {
            let key = metric.0;
            let chart = charts.get_or_create_chart(key);

            chart.update(metric.1);
        }

        self.state.metrics = metrics;
        self.state.score = score;
        self.state.index = index;
        self.state.objective = objective;
        self.state.all_tags = self.state.metrics.tags().cloned().collect();
        self.state.all_tags.sort();
    }
}

impl Dashboard {
    pub(crate) fn ui(f: &mut ratatui::Frame, state: &mut AppState) {
        state.render_count += 1;

        let size = f.area();
        f.buffer_mut().set_style(
            size,
            Style::default().bg(material::GRAY.c900).fg(TEXT_FG_COLOR),
        );

        let base = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(30), Constraint::Fill(1)])
            .split(size);

        Self::render_base_left(f, base[0], state);
        Self::render_metrics_table(f, base[1], state);
    }

    fn render_metrics_table(f: &mut ratatui::Frame, area: Rect, ui_state: &mut AppState) {
        if ui_state.display_tag_filters {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(3 + 20), Constraint::Fill(1)])
                .split(area);

            Self::render_tag_filter_panel(f, chunks[0], ui_state);
            Self::render_metrics_main(f, chunks[1], ui_state);
        } else {
            Self::render_metrics_main(f, area, ui_state);
        }
    }

    fn render_metrics_main(f: &mut ratatui::Frame, area: Rect, ui_state: &mut AppState) {
        let block = Block::bordered();
        let inner = block.inner(area);
        f.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // tabs
                Constraint::Fill(1),   // table
                Constraint::Length(1), // help text
            ])
            .split(inner);

        Self::render_metrics_tabs(f, chunks[0], ui_state.metrics_tab);

        match ui_state.metrics_tab {
            MetricsTab::Time => Self::render_time_table(f, chunks[1], ui_state),
            MetricsTab::Stats => Self::render_stats_table(f, chunks[1], ui_state),
            MetricsTab::Distributions => Self::render_distribution_table(f, chunks[1], ui_state),
        }

        f.render_widget(help_text_widget(), chunks[2]);
    }

    fn render_tag_filter_panel(f: &mut ratatui::Frame, area: Rect, ui_state: &mut AppState) {
        let block = Block::bordered().title(Line::from(" Filter ").centered());
        let inner = block.inner(area);
        f.render_widget(block, area);

        let tags = ui_state
            .all_tags
            .iter()
            .enumerate()
            .map(|(i, tag)| {
                if ui_state.tag_view.contains(&i) {
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

        let tag_line = List::new(tags);
        f.render_widget(tag_line, inner);
    }

    fn render_metrics_tabs(f: &mut ratatui::Frame, area: Rect, active: MetricsTab) {
        use ratatui::text::Span;
        use ratatui::widgets::Tabs;

        let titles = ["Time", "Stats", "Dist"]
            .into_iter()
            .map(|t| Span::styled(format!(" {t} "), Style::default().fg(Color::White)));

        let index = match active {
            MetricsTab::Time => 0,
            MetricsTab::Stats => 1,
            MetricsTab::Distributions => 2,
        };

        let tabs = Tabs::default()
            .titles(titles)
            .select(index)
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .bold();

        f.render_widget(tabs, area);
    }

    fn render_base_left(f: &mut ratatui::Frame, area: Rect, ui_state: &mut AppState) {
        let left = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ])
            .split(area);

        Self::render_engine_base_table(f, left[0], ui_state);
        f.render_widget(
            ui_state.charts().fitness_chart().create_chart_widgets(),
            left[1],
        );

        if let Some(dist_chart) = ui_state.get_chart_by_key(metric_names::SCORES) {
            f.render_widget(dist_chart.create_value_chart_widget(), left[2]);
        }
    }

    fn render_engine_base_table(f: &mut ratatui::Frame, area: Rect, ui_state: &mut AppState) {
        let elapsed = ui_state
            .metrics()
            .time()
            .and_then(|m| m.time_sum())
            .map(fmt_duration)
            .unwrap_or_else(|| "00:00:00.000".to_string());

        let diversity = ui_state
            .metrics()
            .diversity_ratio()
            .and_then(|m| m.value_mean())
            .unwrap_or(0.0);
        let carryover = ui_state
            .metrics()
            .carryover_rate()
            .and_then(|m| m.value_mean())
            .unwrap_or(0.0);
        let unique_members = ui_state
            .metrics()
            .unique_members()
            .and_then(|m| m.value_mean())
            .unwrap_or(0.0);
        let unique_scores = ui_state
            .metrics()
            .unique_scores()
            .and_then(|m| m.value_mean())
            .unwrap_or(0.0);
        let improvements = ui_state
            .metrics()
            .improvements()
            .map(|m| m.count())
            .unwrap_or(0);
        let lifetime_unique = ui_state
            .metrics()
            .lifetime_unique_members()
            .map(|m| m.last_value())
            .unwrap_or(0.0);
        let new_children = ui_state
            .metrics()
            .new_children()
            .and_then(|m| m.value_mean())
            .unwrap_or(0.0);

        let rows = vec![
            Row::new(vec![
                Cell::from("Improvements").bold(),
                Cell::from(improvements.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Diversity").bold(),
                Cell::from(format!("{:.2}%", diversity * 100.0)),
            ]),
            Row::new(vec![
                Cell::from("Carryover").bold(),
                Cell::from(format!("{:.2}%", carryover * 100.0)),
            ]),
            Row::new(vec![
                Cell::from("Unique Members").bold(),
                Cell::from(format!("{:.2}", unique_members)),
            ]),
            Row::new(vec![
                Cell::from("Unique Scores").bold(),
                Cell::from(format!("{:.2}", unique_scores)),
            ]),
            Row::new(vec![
                Cell::from("Phenotypes").bold(),
                Cell::from(format!("{:.2}", lifetime_unique)),
            ]),
            Row::new(vec![
                Cell::from("Children / Gen.").bold(),
                Cell::from(format!("{:.2}", new_children)),
            ]),
        ];

        let engine_state = if ui_state.is_running() {
            "Running".fg(Color::LightGreen).bold()
        } else {
            "Complete".fg(Color::Red).bold()
        };

        let block_title = Line::from(vec![
            "Gen ".fg(Color::Gray).bold(),
            format!("{}", ui_state.index()).fg(Color::LightGreen),
            "| Score ".fg(Color::Gray).bold(),
            format!("{:.4} |", ui_state.score().as_f32()).fg(Color::LightGreen),
            " Time ".fg(Color::Gray).bold(),
            elapsed.clone().fg(Color::LightGreen),
        ])
        .centered();

        let block = Block::bordered().title_top(engine_state);
        let inner = block.inner(area);
        f.render_widget(block, area);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(15), Constraint::Fill(1)])
            .split(inner);

        let engine_table = Table::default()
            .header(header_row(&ENGINE_HEADER_CELLS))
            .rows(striped_rows(rows))
            .widths(&[Constraint::Fill(1), Constraint::Fill(1)]);

        f.render_widget(block_title, layout[0]);
        f.render_widget(engine_table, layout[1]);
    }

    fn render_time_table(f: &mut ratatui::Frame, area: Rect, ui_state: &mut AppState) {
        let row_count = ui_state.time_row_count;
        let selected = ui_state.time_table.selected().unwrap_or(0);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Fill(1)])
            .split(area);

        // --- Left: bar chart of time by metric ---
        let step_metrics = collect_scope_metrics(&ui_state.metrics, MetricScope::Step, ui_state);
        let selector_metrics =
            collect_tagged_metrics(&ui_state.metrics, metric_tags::SELECTOR, ui_state);
        let alterer_metrics =
            collect_tagged_metrics(&ui_state.metrics, metric_tags::ALTERER, ui_state);

        // Create slices
        let slices = step_metrics
            .iter()
            .chain(selector_metrics.iter())
            .chain(alterer_metrics.iter())
            .enumerate()
            .filter_map(|(index, (name, m))| {
                m.time_statistic().map(|time| {
                    let total_ms = time.sum().as_millis() as f64;

                    const MAX_LABEL_LEN: usize = 12;
                    let mut label = (*name).to_string();
                    if label.len() > MAX_LABEL_LEN {
                        label.truncate(MAX_LABEL_LEN - 1);
                        label.push('…');
                    }

                    PieSlice::new(name, total_ms, COLORS[index % COLORS.len()])
                })
            })
            .collect::<Vec<_>>();

        // With customization
        let piechart = PieChart::new(slices)
            .show_legend(true)
            .show_percentages(true)
            .block(Block::bordered())
            .high_resolution(true);

        f.render_widget(piechart, chunks[0]);

        ui_state.time_row_count =
            step_metrics.len() + selector_metrics.len() + alterer_metrics.len();

        let rows = metric_to_time_rows(step_metrics.into_iter())
            .chain(metric_to_time_rows(selector_metrics.into_iter()))
            .chain(metric_to_time_rows(alterer_metrics.into_iter()));

        let table = Table::default()
            .block(Block::bordered())
            .header(header_row(&TIME_HEADER_CELLS))
            .rows(striped_rows(rows))
            .row_highlight_style(Style::default().bg(material::LIGHT_BLUE.c800))
            .widths(&[
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ]);

        render_scrollable_table(
            f,
            chunks[1],
            table,
            &mut ui_state.time_table,
            row_count,
            selected,
        );
    }

    fn render_stats_table(f: &mut ratatui::Frame, area: Rect, ui_state: &mut AppState) {
        let mut items = collect_scope_metrics(&ui_state.metrics, MetricScope::Generation, ui_state);

        let mut alter_metrics =
            collect_tagged_metrics(&ui_state.metrics, metric_tags::ALTERER, ui_state);
        let mut species_metrics =
            collect_tagged_metrics(&ui_state.metrics, metric_tags::SPECIES, ui_state);
        let mut derived_metrics =
            collect_tagged_metrics(&ui_state.metrics, metric_tags::DERIVED, ui_state);

        items.retain(|(name, _)| {
            !alter_metrics.iter().any(|(n, _)| n == name)
                && !species_metrics.iter().any(|(n, _)| n == name)
                && !derived_metrics.iter().any(|(n, _)| n == name)
        });

        let ordered = alter_metrics
            .drain(..)
            .chain(species_metrics.drain(..))
            .chain(items.drain(..))
            .chain(derived_metrics.drain(..))
            .collect::<Vec<_>>();

        let prev_row_count = ui_state.stats_row_count;
        let selected = ui_state.stats_table.selected().unwrap_or(0);

        ui_state.stats_row_count = ordered.len();

        let rows = metrics_into_stat_rows(ordered.iter().copied());

        let table = Table::default()
            .block(Block::bordered())
            .header(header_row(&STAT_HEADER_CELLS))
            .rows(striped_rows(rows))
            .row_highlight_style(Style::default().bg(material::LIGHT_BLUE.c800))
            .widths(&[
                Constraint::Length(22),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ]);

        let display_any_chart = ui_state.display_metric_charts || ui_state.display_metric_means;

        if display_any_chart {
            let v_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Fill(3),   // table area
                    Constraint::Length(7), // detail panel
                ])
                .split(area);

            render_scrollable_table(
                f,
                v_chunks[0],
                table,
                &mut ui_state.stats_table,
                prev_row_count,
                selected,
            );

            if !ordered.is_empty() && v_chunks[1].height > 3 {
                let maybe_chart =
                    ui_state.get_chart_by_key(ordered[selected.min(ordered.len() - 1)].0);
                if let Some(chart) = maybe_chart {
                    if ui_state.display_metric_charts && ui_state.display_metric_means {
                        f.render_widget(chart.create_chart_widgets(), v_chunks[1]);
                    } else if ui_state.display_metric_means {
                        f.render_widget(chart.create_mean_chart_widget().unwrap(), v_chunks[1]);
                    } else {
                        f.render_widget(chart.create_value_chart_widget(), v_chunks[1]);
                    }
                }
            }
        } else {
            render_scrollable_table(
                f,
                area,
                table,
                &mut ui_state.stats_table,
                prev_row_count,
                selected,
            );
        }
    }

    fn render_distribution_table(f: &mut ratatui::Frame, area: Rect, ui_state: &mut AppState) {
        let prev_row_count = ui_state.distribution_row_count;
        let selected = ui_state.distribution_table.selected().unwrap_or(0);

        let ordered: Vec<(&'static str, &Metric)> = ui_state
            .metrics
            .iter_tagged(metric_tags::DISTRIBUTION)
            .filter(|(_, m)| ui_state.metric_has_tags(m))
            .collect();

        ui_state.distribution_row_count = ordered.len();

        let dist_rows = metrics_into_dist_rows(ordered.iter().copied()).collect::<Vec<_>>();
        let len_rows = dist_rows.len();
        let table = Table::default()
            .block(Block::bordered())
            .header(header_row(&DISTRIBUTION_HEADER_CELLS))
            .rows(striped_rows(dist_rows))
            .row_highlight_style(Style::default().bg(material::LIGHT_BLUE.c800))
            .widths(&[
                Constraint::Length(22),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ]);

        let display_any_chart = ui_state.display_metric_charts || ui_state.display_metric_means;

        if display_any_chart {
            let v_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Fill(3), Constraint::Length(7)])
                .split(area);

            render_scrollable_table(
                f,
                v_chunks[0],
                table,
                &mut ui_state.distribution_table,
                prev_row_count,
                selected,
            );

            if len_rows > 0 && v_chunks[1].height > 3 {
                let maybe_chart = ui_state.get_chart_by_key(ordered[selected.min(len_rows - 1)].0);
                if let Some(chart) = maybe_chart {
                    if ui_state.display_metric_charts && ui_state.display_metric_means {
                        f.render_widget(chart.create_chart_widgets(), v_chunks[1]);
                    } else if ui_state.display_metric_means {
                        f.render_widget(chart.create_mean_chart_widget().unwrap(), v_chunks[1]);
                    } else {
                        f.render_widget(chart.create_value_chart_widget(), v_chunks[1]);
                    }
                }
            }
        } else {
            render_scrollable_table(
                f,
                area,
                table,
                &mut ui_state.distribution_table,
                prev_row_count,
                selected,
            );
        }
    }
}

fn render_scrollable_table(
    f: &mut ratatui::Frame,
    area: Rect,
    table: Table,
    state: &mut TableState,
    row_count: usize,
    selected: usize,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),   // table
            Constraint::Length(1), // scrollbar column
        ])
        .split(area);

    f.render_stateful_widget(table, chunks[0], state);

    if row_count > chunks[0].height as usize {
        let mut scrollbar_state = ScrollbarState::new(row_count).position(selected);

        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .track_style(Style::default().fg(Color::DarkGray))
            .thumb_style(Style::default().fg(Color::LightGreen));

        f.render_stateful_widget(scrollbar, chunks[1], &mut scrollbar_state);
    }
}

// --------- metric helpers ---------

fn collect_scope_metrics<'a>(
    metrics: &'a MetricSet,
    scope: MetricScope,
    ui_state: &AppState,
) -> Vec<(&'static str, &'a Metric)> {
    let mut items: Vec<_> = metrics
        .iter_scope(scope)
        .filter(|(_, m)| ui_state.metric_has_tags(m))
        .collect();
    items.sort_by(|a, b| a.0.cmp(b.0));
    items
}

fn collect_tagged_metrics<'a>(
    metrics: &'a MetricSet,
    tag: &'static str,
    ui_state: &AppState,
) -> Vec<(&'static str, &'a Metric)> {
    let mut items: Vec<_> = metrics
        .iter_tagged(tag)
        .filter(|(_, m)| ui_state.metric_has_tags(m))
        .collect();
    items.sort_by(|a, b| a.0.cmp(b.0));
    items
}

/// --- Row Builders ---

fn metric_to_time_rows<'a>(
    metrics: impl Iterator<Item = (&'static str, &'a Metric)>,
) -> impl Iterator<Item = Row<'a>> {
    metrics.filter_map(|(name, m)| {
        if let Some(time) = m.time_statistic() {
            let mean = fmt_duration(time.mean());
            let min = fmt_duration(time.min());
            let max = fmt_duration(time.max());
            let total = fmt_duration(time.sum());

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
        if let Some(stat) = m.statistic() {
            Some(Row::new(vec![
                Cell::from(name.to_string()),
                Cell::from(format!("{:.2}", stat.min())),
                Cell::from(format!("{:.2}", stat.max())),
                Cell::from(format!("{:.2}", stat.mean())),
                Cell::from(format!("{:.2}", stat.sum())),
                Cell::from(format!("{:.2}", stat.std_dev())),
                Cell::from(format!("{:.2}", stat.variance())),
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
        if let Some(dist) = m.distribution() {
            Some(Row::new(vec![
                Cell::from(name.to_string()),
                Cell::from(format!("{:.3}", dist.percentile(0.0))),
                Cell::from(format!("{:.3}", dist.percentile(25.0))),
                Cell::from(format!("{:.3}", dist.percentile(50.0))),
                Cell::from(format!("{:.3}", dist.percentile(75.0))),
                Cell::from(format!("{:.3}", dist.percentile(100.0))),
                Cell::from(format!("{}", dist.count())),
                Cell::from(format!("{:.2}", dist.standard_deviation())),
                Cell::from(format!("{:.2}", dist.variance())),
                Cell::from(format!("{:.2}", dist.skewness())),
                Cell::from(format!("{:.2}", dist.entropy())),
            ]))
        } else {
            None
        }
    })
}

fn striped_rows<'a>(rows: impl IntoIterator<Item = Row<'a>>) -> impl Iterator<Item = Row<'a>> {
    rows.into_iter().enumerate().map(|(i, row)| {
        let bg = if i % 2 == 0 {
            NORMAL_ROW_BG
        } else {
            ALT_ROW_BG_COLOR
        };
        row.style(Style::default().bg(bg))
    })
}

fn header_row<'a>(cols: &'a [&str]) -> Row<'a> {
    Row::new(cols.iter().copied().map(Cell::from))
        .height(1)
        .style(Style::default().bold().underlined().fg(Color::White))
}

fn help_text_widget() -> Paragraph<'static> {
    let help_text = Line::from(vec![
        Span::from("Use "),
        "[j/k]".fg(Color::LightGreen).bold(),
        Span::from(" to navigate, "),
        "[t]".fg(Color::LightGreen).bold(),
        Span::from(" for Time, "),
        "[s]".fg(Color::LightGreen).bold(),
        Span::from(" for Stats, "),
        "[d]".fg(Color::LightGreen).bold(),
        Span::from(" for Distributions, "),
        "[f]".fg(Color::LightGreen).bold(),
        Span::from(" to toggle filters."),
        "[c]".fg(Color::LightGreen).bold(),
        Span::from(" to toggle rolling."),
        "[m]".fg(Color::LightGreen).bold(),
        Span::from(" to toggle means."),
    ])
    .centered();
    Paragraph::new(help_text)
}
