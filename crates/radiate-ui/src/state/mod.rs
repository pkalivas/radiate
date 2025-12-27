use crate::chart::RollingChart;
use crate::widgets::num_pairs;
use radiate_engines::stats::TagKind;
use radiate_engines::{
    Chromosome, Front, Metric, MetricSet, Objective, Optimize, Phenotype, Score,
};
use ratatui::widgets::{ListState, ScrollbarState, TableState};
use std::time::{Duration, Instant};

pub mod chart;
pub use chart::{ChartState, ChartType};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PanelId {
    Filters,
    Metrics,
    Help,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RunningState {
    pub engine: bool,
    pub ui: bool,
    pub paused: bool,
}

pub struct DisplayState {
    pub show_tag_filters: bool,
    pub show_mini_chart: bool,
    pub show_mini_chart_mean: bool,
    pub show_help: bool,
    pub focus_panel: PanelId,
    pub modal_panel: Option<PanelId>,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum MetricsTab {
    #[default]
    Time,
    Stats,
}

impl MetricsTab {
    pub fn next(self) -> Self {
        match self {
            MetricsTab::Stats => MetricsTab::Time,
            MetricsTab::Time => MetricsTab::Stats,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            MetricsTab::Stats => MetricsTab::Time,
            MetricsTab::Time => MetricsTab::Stats,
        }
    }
}

pub struct ObjectiveState {
    pub objective: Objective,
    pub charts_visible: usize,
    pub chart_start_index: usize,
}

pub struct AppState<C: Chromosome> {
    pub front: Option<Front<Phenotype<C>>>,
    pub last_render: Option<Instant>,
    pub render_interval: Duration,
    pub chart_state: ChartState,
    pub metrics_tab: MetricsTab,

    pub running: RunningState,
    pub display: DisplayState,

    pub filter_state: AppFilterState,

    pub objective_state: ObjectiveState,
    pub metrics: MetricSet,
    pub index: usize,
    pub score: Score,

    pub time_table: AppTableState,
    pub stats_table: AppTableState,
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
            self.display.focus_panel = PanelId::Metrics;
        } else {
            self.display.focus_panel = PanelId::Filters;
        }
    }

    pub fn toggle_help(&mut self) {
        self.display.show_help = !self.display.show_help;
        if self.display.show_help {
            self.display.modal_panel = Some(PanelId::Help);
        } else {
            self.display.modal_panel = None;
        }
    }

    pub fn expand_objective_pairs(&mut self) {
        self.objective_state.charts_visible = self
            .objective_state
            .charts_visible
            .saturating_add(1)
            .min(num_pairs(self.objective_state.objective.dims()));
    }

    pub fn shrink_objective_pairs(&mut self) {
        if self.objective_state.charts_visible > 1 {
            self.objective_state.charts_visible -= 1;
        }
    }

    pub fn clear_filters(&mut self) {
        if self.display.show_help {
            self.display.show_help = false;
            self.display.modal_panel = None;
        }

        if !self.display.show_tag_filters {
            return;
        }

        self.filter_state.tag_view.clear();
    }

    pub fn next_objective_pair_page(&mut self) {
        let step = self.objective_state.charts_visible.max(1);
        let total = num_pairs(self.objective_state.objective.dims());
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

    pub fn get_chart_by_key(
        &self,
        key: &'static str,
        chart_type: ChartType,
    ) -> Option<&RollingChart> {
        self.chart_state.get_by_key(key, chart_type)
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

    pub fn is_engine_paused(&self) -> bool {
        self.running.paused
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

    pub fn chart_state(&self) -> &ChartState {
        &self.chart_state
    }

    pub fn chart_state_mut(&mut self) -> &mut ChartState {
        &mut self.chart_state
    }

    pub fn next_metrics_tab(&mut self) {
        self.metrics_tab = self.metrics_tab.next();
    }

    pub fn previous_metrics_tab(&mut self) {
        self.metrics_tab = self.metrics_tab.previous();
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
        if self.display.focus_panel == PanelId::Filters {
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
        if self.display.focus_panel == PanelId::Filters {
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
        if self.display.focus_panel != PanelId::Filters {
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

            running: RunningState {
                engine: false,
                ui: true,
                paused: false,
            },
            display: DisplayState {
                show_tag_filters: false,
                show_mini_chart: true,
                show_mini_chart_mean: false,
                show_help: false,
                focus_panel: PanelId::Metrics,
                modal_panel: None,
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
            metrics: MetricSet::new(),
            index: 0,
            score: Score::default(),
        }
    }
}

pub struct AppFilterState {
    pub tag_list_filter_state: ListState,
    pub tag_view: Vec<usize>,
    pub all_tags: Vec<TagKind>,
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

        if (self.prev_row_count == 0 && self.row_count > 0)
            || (self.selected_row < self.prev_row_count && self.selected_row >= self.row_count)
        {
            self.selected_row = 0;
            self.state.select(Some(0));
        }

        if self.selected_row >= current_len && current_len > 0 {
            self.selected_row = current_len - 1;
            self.state.select(Some(self.selected_row));
        }

        self.selected_metric = items.get(self.selected_row).map(|(name, _)| *name);
    }
}
