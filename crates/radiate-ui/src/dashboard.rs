use crate::{
    chart::ChartData,
    state::{DashboardState, MetricsTab},
};
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
        Bar, BarChart, BarGroup, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, TableState,
    },
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize, palette::material},
    symbols::{self},
    text::Line,
    widgets::{Axis, Block, Cell, Chart, Dataset, GraphType, Row, Table},
};
use std::{
    io,
    time::{Duration, Instant},
};

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

pub(crate) struct Dashboard {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    state: DashboardState,
}

impl Dashboard {
    pub fn new(render_interval: Duration) -> Self {
        Self {
            terminal: ratatui::init(),
            state: DashboardState {
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

    pub fn set_tag_filter_by_index(&mut self, index: usize) {
        self.state.set_tag_filter_by_index(index);
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
            .scores_mut()
            .add_value((index as f64, score.as_f32() as f64));

        if let Some(score) = metrics.get(metric_names::SCORES) {
            charts.scores_mean_mut().add_value((
                index as f64,
                score.distribution_mean().unwrap_or(0.0) as f64,
            ));

            if let Some(dist) = score.distribution() {
                charts.score_dist_mut().set_values(dist.last_sequence());
            }
        }

        if let Some(diversity) = metrics.get(metric_names::DIVERSITY_RATIO) {
            charts
                .diversity_mut()
                .add_value((index as f64, diversity.last_value() as f64));
        }

        if let Some(carryover) = metrics.get(metric_names::CARRYOVER_RATE) {
            charts
                .carryover_mut()
                .add_value((index as f64, carryover.last_value() as f64));
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
    pub(crate) fn ui(f: &mut ratatui::Frame, state: &mut DashboardState) {
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

    fn render_metrics_table(f: &mut ratatui::Frame, area: Rect, ui_state: &mut DashboardState) {
        if ui_state.display_tag_filters {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(3 + 20), Constraint::Fill(1)])
                .split(area);

            Self::render_tag_filter_panel(f, chunks[0], ui_state);

            let block = Block::bordered();
            let inner = block.inner(chunks[1]);
            f.render_widget(block, chunks[1]);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2),
                    Constraint::Fill(1),
                    Constraint::Length(1),
                ])
                .split(inner);

            Self::render_metrics_tabs(f, chunks[0], ui_state.metrics_tab);

            match ui_state.metrics_tab {
                MetricsTab::Time => Self::render_time_table(f, chunks[1], ui_state),
                MetricsTab::Stats => Self::render_stats_table(f, chunks[1], ui_state),
                MetricsTab::Distributions => {
                    Self::render_distribution_table(f, chunks[1], ui_state)
                }
            }

            f.render_widget(help_text_widget(), chunks[2]);
        } else {
            let block = Block::bordered();
            let inner = block.inner(area);
            f.render_widget(block, area);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2),
                    Constraint::Fill(1),
                    Constraint::Length(1),
                ])
                .split(inner);

            Self::render_metrics_tabs(f, chunks[0], ui_state.metrics_tab);

            match ui_state.metrics_tab {
                MetricsTab::Time => Self::render_time_table(f, chunks[1], ui_state),
                MetricsTab::Stats => Self::render_stats_table(f, chunks[1], ui_state),
                MetricsTab::Distributions => {
                    Self::render_distribution_table(f, chunks[1], ui_state)
                }
            }

            f.render_widget(help_text_widget(), chunks[2]);
        }
    }

    fn render_tag_filter_panel(f: &mut ratatui::Frame, area: Rect, ui_state: &mut DashboardState) {
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

    fn render_base_left(f: &mut ratatui::Frame, area: Rect, ui_state: &mut DashboardState) {
        let left = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ])
            .split(area);

        Self::render_engine_base_table(f, left[0], ui_state);
        Self::render_line_charts(
            f,
            left[1],
            vec![&ui_state.score_chart(), &ui_state.score_mean()],
        );

        Self::render_line_charts(
            f,
            left[2],
            vec![&ui_state.score_dist_chart()],
            // vec![&ui_state.diversity_chart(), &ui_state.carryover_chart()],
        );
    }

    fn render_engine_base_table(f: &mut ratatui::Frame, area: Rect, ui_state: &mut DashboardState) {
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

        let block_title = Line::from(vec![
            " Gen ".fg(Color::Gray).bold(),
            format!("{}", ui_state.index()).fg(Color::LightGreen),
            "| Score ".fg(Color::Gray).bold(),
            format!("{:.4} |", ui_state.score().as_f32()).fg(Color::LightGreen),
            " Time ".fg(Color::Gray).bold(),
            elapsed.clone().fg(Color::LightGreen),
        ])
        .centered();

        let block = Block::bordered();
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

    fn render_time_table(f: &mut ratatui::Frame, area: Rect, ui_state: &mut DashboardState) {
        let row_count = ui_state.time_row_count;
        let selected = ui_state.time_table.selected().unwrap_or(0);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Fill(1)])
            .split(area);

        let mut step_metrics = ui_state
            .metrics
            .iter_scope(MetricScope::Step)
            .filter(|(_, m)| ui_state.metric_has_tags(m))
            .collect::<Vec<_>>();
        let mut selector_metrics = ui_state
            .metrics
            .iter_tagged(metric_tags::SELECTOR)
            .filter(|(_, m)| ui_state.metric_has_tags(m))
            .collect::<Vec<_>>();
        let mut alterer_metrics = ui_state
            .metrics
            .iter_tagged(metric_tags::ALTERER)
            .filter(|(_, m)| ui_state.metric_has_tags(m))
            .collect::<Vec<_>>();

        step_metrics.sort_by(|a, b| a.0.cmp(b.0));
        selector_metrics.sort_by(|a, b| a.0.cmp(b.0));
        alterer_metrics.sort_by(|a, b| a.0.cmp(b.0));

        let bars = step_metrics
            .iter()
            .chain(selector_metrics.iter())
            .chain(alterer_metrics.iter())
            .filter_map(|(name, m)| {
                m.time_statistic().map(|time| {
                    let total_ms = time.sum().as_millis() as u64;

                    const MAX_LABEL_LEN: usize = 12;
                    let mut label = (*name).to_string();
                    if label.len() > MAX_LABEL_LEN {
                        label.truncate(MAX_LABEL_LEN - 1);
                        label.push('…');
                    }

                    Bar::default()
                        .value(total_ms)
                        .label(label.into())
                        .text_value(format!("{} ms", total_ms))
                })
            })
            .collect::<Vec<_>>();

        let group = BarGroup::default().bars(&bars);

        let bar_chart = BarChart::default()
            .data(group)
            .bar_width(1)
            .bar_gap(1)
            .direction(Direction::Horizontal)
            .block(Block::bordered());

        f.render_widget(bar_chart, chunks[0]);

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

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),   // table
                Constraint::Length(1), // scrollbar column
            ])
            .split(chunks[1]);

        f.render_stateful_widget(table, chunks[0], &mut ui_state.time_table);

        if row_count > chunks[0].height as usize {
            let mut scrollbar_state = ScrollbarState::new(row_count).position(selected);

            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .track_style(Style::default().fg(Color::DarkGray))
                .thumb_style(Style::default().fg(Color::LightGreen));

            f.render_stateful_widget(scrollbar, chunks[1], &mut scrollbar_state);
        }
    }

    fn render_stats_table(f: &mut ratatui::Frame, area: Rect, ui_state: &mut DashboardState) {
        let mut items: Vec<_> = ui_state
            .metrics
            .iter_scope(MetricScope::Generation)
            .filter(|(_, m)| ui_state.metric_has_tags(m))
            .collect();
        items.sort_by(|a, b| a.0.cmp(b.0));

        let mut alter_metrics = ui_state
            .metrics
            .iter_tagged(metric_tags::ALTERER)
            .filter(|(_, m)| ui_state.metric_has_tags(m))
            .collect::<Vec<_>>();
        let mut species_metrics = ui_state
            .metrics
            .iter_tagged(metric_tags::SPECIES)
            .filter(|(_, m)| ui_state.metric_has_tags(m))
            .collect::<Vec<_>>();
        let mut derived_metrics = ui_state
            .metrics
            .iter_tagged(metric_tags::DERIVED)
            .filter(|(_, m)| ui_state.metric_has_tags(m))
            .collect::<Vec<_>>();

        alter_metrics.sort_by(|a, b| a.0.cmp(b.0));
        species_metrics.sort_by(|a, b| a.0.cmp(b.0));
        derived_metrics.sort_by(|a, b| a.0.cmp(b.0));
        items.retain(|(name, _)| {
            !alter_metrics.iter().any(|(n, _)| n == name)
                && !species_metrics.iter().any(|(n, _)| n == name)
                && !derived_metrics.iter().any(|(n, _)| n == name)
        });

        let row_count = ui_state.stats_row_count;
        let selected = ui_state.stats_table.selected().unwrap_or(0);

        ui_state.stats_row_count =
            alter_metrics.len() + species_metrics.len() + items.len() + derived_metrics.len();

        let rows = metrics_into_stat_rows(alter_metrics.into_iter())
            .chain(metrics_into_stat_rows(species_metrics.into_iter()))
            .chain(metrics_into_stat_rows(items.into_iter()))
            .chain(metrics_into_stat_rows(derived_metrics.into_iter()));

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

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),   // table
                Constraint::Length(1), // scrollbar column
            ])
            .split(area);

        f.render_stateful_widget(table, chunks[0], &mut ui_state.stats_table);

        if row_count > chunks[0].height as usize {
            let mut scrollbar_state = ScrollbarState::new(row_count).position(selected);

            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .track_style(Style::default().fg(Color::DarkGray))
                .thumb_style(Style::default().fg(Color::LightGreen));

            f.render_stateful_widget(scrollbar, chunks[1], &mut scrollbar_state);
        }
    }

    fn render_distribution_table(
        f: &mut ratatui::Frame,
        area: Rect,
        ui_state: &mut DashboardState,
    ) {
        let row_count = ui_state.distribution_row_count;
        let selected = ui_state.distribution_table.selected().unwrap_or(0);

        let distibutions = metrics_into_dist_rows(
            ui_state
                .metrics
                .iter_tagged(metric_tags::DISTRIBUTION)
                .filter(|(_, m)| ui_state.metric_has_tags(m)),
        )
        .collect::<Vec<_>>();
        ui_state.distribution_row_count = distibutions.len();
        let table = Table::default()
            .block(Block::bordered())
            .header(header_row(&DISTRIBUTION_HEADER_CELLS))
            .rows(striped_rows(distibutions))
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

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),   // table
                Constraint::Length(1), // scrollbar column
            ])
            .split(area);

        f.render_stateful_widget(table, chunks[0], &mut ui_state.distribution_table);

        if row_count > chunks[0].height as usize {
            let mut scrollbar_state = ScrollbarState::new(row_count).position(selected);

            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .track_style(Style::default().fg(Color::DarkGray))
                .thumb_style(Style::default().fg(Color::LightGreen));

            f.render_stateful_widget(scrollbar, chunks[1], &mut scrollbar_state);
        }
    }

    fn render_line_charts(frame: &mut ratatui::Frame, area: Rect, charts: Vec<&ChartData>) {
        let (min_x, max_x) = charts
            .iter()
            .fold((f64::MAX, f64::MIN), |(min_x, max_x), dim| {
                (min_x.min(dim.min_x()), max_x.max(dim.max_x()))
            });

        let (min_y, max_y) = charts
            .iter()
            .fold((f64::MAX, f64::MIN), |(min_y, max_y), dim| {
                (min_y.min(dim.min_y()), max_y.max(dim.max_y()))
            });

        let datasets = charts
            .iter()
            .map(|dim| {
                Dataset::default()
                    .name(dim.name())
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(dim.color()))
                    .graph_type(GraphType::Line)
                    .data(dim.values())
            })
            .collect::<Vec<_>>();

        let chart = Chart::new(datasets)
            .block(Block::bordered())
            .x_axis(
                Axis::default()
                    .style(Style::default().gray())
                    .bounds([min_x, max_x])
                    .labels([
                        "0".bold(),
                        format!("{}", (charts[0].values().len() / 2)).into(),
                        format!("{}", charts[0].values().len() - 1).bold().into(),
                    ]),
            )
            .y_axis(
                Axis::default()
                    .style(Style::default().gray())
                    .bounds([min_y, max_y])
                    .labels([
                        format!("{:.2}", min_y).bold(),
                        format!("{:.2}", ((min_y + max_y) / 2.0)).into(),
                        format!("{:.2}", max_y).bold(),
                    ]),
            )
            .hidden_legend_constraints((Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)));

        frame.render_widget(chart, area);
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
        Span::from(" Metrics Table "),
        Span::from("| Use "),
        "[j/k]".fg(Color::LightGreen).bold(),
        Span::from(" to navigate, "),
        "[t]".fg(Color::LightGreen).bold(),
        Span::from(" for Time, "),
        "[s]".fg(Color::LightGreen).bold(),
        Span::from(" for Stats, "),
        "[d]".fg(Color::LightGreen).bold(),
        Span::from(" for Distributions. "),
    ])
    .centered();
    Paragraph::new(help_text)
}
