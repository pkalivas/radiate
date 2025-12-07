use crate::chart::ChartData;
use crate::widgets::num_pairs;
use radiate_engines::{
    Chromosome, Front, Metric, MetricSet, Objective, Optimize, Phenotype, Score, stats::Tag,
};
use ratatui::widgets::{ListState, ScrollbarState, TableState};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct RunningState {
    pub engine: bool,
    pub ui: bool,
}

pub(crate) struct DisplayState {
    pub show_tag_filters: bool,
    pub show_mini_chart: bool,
    pub show_mini_chart_mean: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum MetricsTab {
    #[default]
    Time,
    Stats,
    Distributions,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PanelTab {
    Filter,
    Metrics,
}

pub struct ChartState {
    fitness: ChartData,
    keyed: HashMap<&'static str, ChartData>,
}

impl ChartState {
    pub fn new() -> Self {
        Self {
            fitness: ChartData::with_capacity(1000).with_name("Score"),
            keyed: HashMap::new(),
        }
    }

    pub fn fitness_chart(&self) -> &ChartData {
        &self.fitness
    }

    pub fn fitness_chart_mut(&mut self) -> &mut ChartData {
        &mut self.fitness
    }

    pub fn get_by_key(&self, key: &'static str) -> Option<&ChartData> {
        self.keyed.get(key)
    }

    pub fn get_or_create_chart(&mut self, key: &'static str) -> &mut ChartData {
        self.keyed
            .entry(key)
            .or_insert_with(|| ChartData::new().with_metric_name(key))
    }
}

pub struct ObjectiveState {
    pub objective: Objective,
    pub charts_visible: usize,
    pub chart_start_index: usize,
}

pub(crate) struct AppState<C: Chromosome> {
    pub front: Option<Front<Phenotype<C>>>,
    pub last_render: Option<Instant>,
    pub render_interval: Duration,
    pub chart_state: ChartState,
    pub metrics_tab: MetricsTab,
    pub panel_tab: PanelTab,

    pub running: RunningState,
    pub display: DisplayState,

    pub filter_state: AppFilterState,

    pub objective_state: ObjectiveState,
    pub metrics: MetricSet,
    pub index: usize,
    pub score: Score,

    pub time_table: AppTableState,
    pub stats_table: AppTableState,
    pub distribution_table: AppTableState,
}

impl<C: Chromosome> AppState<C> {
    pub fn render_interval(&self) -> Duration {
        self.render_interval
    }

    pub fn set_tag_filter_by_index(&mut self, index: usize) {
        if !self.display.show_tag_filters {
            return;
        }

        if self.filter_state.tag_view.contains(&index) {
            self.filter_state.tag_view.retain(|&i| i != index);
        } else {
            if index < self.filter_state.all_tags.len() {
                self.filter_state.tag_view.push(index);
            } else {
                self.filter_state.tag_view.retain(|&i| i != index);
            }
        }
    }

    pub fn toggle_mini_chart(&mut self) {
        self.display.show_mini_chart = !self.display.show_mini_chart;
    }

    pub fn toggle_mini_chart_mean(&mut self) {
        self.display.show_mini_chart_mean = !self.display.show_mini_chart_mean;
    }

    pub fn toggle_show_tag_filters(&mut self) {
        self.display.show_tag_filters = !self.display.show_tag_filters;
        if !self.display.show_tag_filters {
            self.panel_tab = PanelTab::Metrics;
        } else {
            self.panel_tab = PanelTab::Filter;
        }
    }

    pub fn expand_objective_pairs(&mut self) {
        self.objective_state.charts_visible = self
            .objective_state
            .charts_visible
            .saturating_add(1)
            .min(num_pairs(self.objective_state.objective.dimensions()));
    }

    pub fn shrink_objective_pairs(&mut self) {
        if self.objective_state.charts_visible > 1 {
            self.objective_state.charts_visible -= 1;
        }
    }

    pub fn clear_tag_filters(&mut self) {
        if !self.display.show_tag_filters {
            return;
        }

        self.filter_state.tag_view.clear();
    }

    pub fn next_objective_pair_page(&mut self) {
        let step = self.objective_state.charts_visible.max(1);
        let total = num_pairs(self.objective_state.objective.dimensions());
        let current = self.objective_state.chart_start_index;
        if current + step < total {
            self.objective_state.chart_start_index += step;
        }
    }

    pub fn previous_objective_pair_page(&mut self) {
        let step = self.objective_state.charts_visible.max(1);
        let current = self.objective_state.chart_start_index;
        if current >= step {
            self.objective_state.chart_start_index -= step;
        } else {
            self.objective_state.chart_start_index = 0;
        }
    }

    pub fn set_metrics_tab(&mut self, tab: MetricsTab) {
        self.metrics_tab = tab;
    }

    pub fn get_chart_by_key(&self, key: &'static str) -> Option<&ChartData> {
        self.chart_state.get_by_key(key)
    }

    pub fn metrics(&self) -> &MetricSet {
        &self.metrics
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn score(&self) -> &Score {
        &self.score
    }

    pub fn is_engine_running(&self) -> bool {
        self.running.engine
    }

    pub fn display_any_mini_chart(&self) -> bool {
        self.display.show_mini_chart || self.display.show_mini_chart_mean
    }

    pub fn display_mini_chart(&self) -> bool {
        self.display.show_mini_chart
    }

    pub fn display_mini_chart_mean(&self) -> bool {
        self.display.show_mini_chart_mean
    }

    pub fn charts(&self) -> &ChartState {
        &self.chart_state
    }

    pub fn charts_mut(&mut self) -> &mut ChartState {
        &mut self.chart_state
    }

    pub fn metric_has_tags(&self, metric: &Metric) -> bool {
        if self.filter_state.tag_view.is_empty() {
            true
        } else {
            for &tag_index in &self.filter_state.tag_view {
                if let Some(tag) = self.filter_state.all_tags.get(tag_index) {
                    if metric.contains_tag(tag) {
                        return true;
                    }
                }
            }

            false
        }
    }

    pub fn move_selection_down(&mut self) {
        if self.panel_tab == PanelTab::Filter {
            if self.filter_state.all_tags.is_empty() {
                return;
            }

            let last_index = self.filter_state.all_tags.len() - 1;
            if self.filter_state.selected_row >= last_index {
                self.filter_state.selected_row = 0;
            } else {
                self.filter_state.selected_row += 1;
            }
            return;
        }

        let state = match self.metrics_tab {
            MetricsTab::Time => &mut self.time_table,
            MetricsTab::Stats => &mut self.stats_table,
            MetricsTab::Distributions => &mut self.distribution_table,
        };

        if state.row_count == 0 {
            return;
        }

        if let Some(i) = state.state.selected() {
            let next = if i + 1 >= state.row_count { 0 } else { i + 1 };

            state.state.select(Some(next));
            state.selected_row = next;
            state.scroll_bar = state.scroll_bar.as_mut().map(|sb| sb.position(next));
        } else {
            state.state.select(Some(0));
            state.selected_row = 0;
            state.scroll_bar = state.scroll_bar.as_mut().map(|sb| sb.position(0));
        }
    }

    pub fn move_selection_up(&mut self) {
        if self.panel_tab == PanelTab::Filter {
            if self.filter_state.all_tags.is_empty() {
                return;
            }

            let last_index = self.filter_state.all_tags.len() - 1;
            if self.filter_state.selected_row == 0 {
                self.filter_state.selected_row = last_index;
            } else {
                self.filter_state.selected_row -= 1;
            }

            return;
        }

        let state = match self.metrics_tab {
            MetricsTab::Time => &mut self.time_table,
            MetricsTab::Stats => &mut self.stats_table,
            MetricsTab::Distributions => &mut self.distribution_table,
        };

        if state.row_count == 0 {
            return;
        }

        if let Some(i) = state.state.selected() {
            let next = if i == 0 {
                state.row_count - 1 // wrap to last
            } else {
                i - 1
            };

            state.state.select(Some(next));
            state.selected_row = next;
            state.scroll_bar = state.scroll_bar.as_mut().map(|sb| sb.position(next));
        } else {
            let last = state.row_count - 1;
            state.state.select(Some(last));
            state.selected_row = last;
            state.scroll_bar = state.scroll_bar.as_mut().map(|sb| sb.position(last));
        }
    }

    pub fn toggle_tag_filter_selection(&mut self) {
        if self.panel_tab != PanelTab::Filter {
            return;
        }

        let selected_index = self.filter_state.selected_row;
        if self.filter_state.tag_view.contains(&selected_index) {
            self.filter_state.tag_view.retain(|&i| i != selected_index);
        } else {
            if selected_index < self.filter_state.all_tags.len() {
                self.filter_state.tag_view.push(selected_index);
            } else {
                self.filter_state.tag_view.retain(|&i| i != selected_index);
            }
        }
    }
}

impl<C: Chromosome> Default for AppState<C> {
    fn default() -> Self {
        Self {
            front: None,
            last_render: None,
            render_interval: Duration::from_millis(500),
            chart_state: ChartState::new(),
            metrics_tab: MetricsTab::Stats,
            panel_tab: PanelTab::Metrics,

            running: RunningState {
                engine: false,
                ui: true,
            },
            display: DisplayState {
                show_tag_filters: false,
                show_mini_chart: true,
                show_mini_chart_mean: false,
            },

            objective_state: ObjectiveState {
                objective: Objective::Single(Optimize::Maximize),
                charts_visible: 2,
                chart_start_index: 0,
            },

            filter_state: AppFilterState {
                tag_list_filter_state: ListState::default(),
                tag_view: Vec::new(),
                all_tags: Vec::new(),
                selected_row: 0,
            },

            time_table: AppTableState::new(),
            stats_table: AppTableState::new(),
            distribution_table: AppTableState::new(),
            metrics: MetricSet::new(),
            index: 0,
            score: Score::default(),
        }
    }
}

pub struct AppFilterState {
    pub tag_list_filter_state: ListState,
    pub tag_view: Vec<usize>,
    pub all_tags: Vec<Tag>,
    pub selected_row: usize,
}

pub struct AppTableState {
    pub state: TableState,
    pub scroll_bar: Option<ScrollbarState>,
    pub selected_metric: Option<&'static str>,
    pub selected_row: usize,
    pub row_count: usize,
    pub prev_row_count: usize,
}

impl AppTableState {
    pub fn new() -> Self {
        Self {
            state: TableState::default(),
            scroll_bar: None,
            selected_row: 0,
            row_count: 0,
            prev_row_count: 0,
            selected_metric: None,
        }
    }

    pub fn update_rows(&mut self, items: &[(&'static str, &Metric)]) {
        let current_len = items.len();
        self.prev_row_count = self.row_count;
        self.row_count = current_len;

        if self.selected_row >= current_len && current_len > 0 {
            self.selected_row = current_len - 1;
            self.state.select(Some(self.selected_row));
        }

        self.selected_metric = items.get(self.selected_row).map(|(name, _)| *name);
    }
}
