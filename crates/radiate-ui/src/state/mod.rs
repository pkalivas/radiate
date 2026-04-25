pub mod chart;

use crate::chart::RollingLineChart;
use crate::widgets::num_pairs;
use radiate_engines::species::SpeciesId;
use radiate_engines::{
    Chromosome, Front, Metric, MetricSet, Objective, Optimize, Phenotype, Score, SpeciesSnapshot,
};
use ratatui::widgets::{Block, ScrollbarState, TableState};
use std::time::{Duration, Instant};
use tui_piechart::border_style;

pub use chart::{ChartState, LineChartType};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PanelId {
    EngineStatus,
    FitnessChart,

    MetricModal,
    Search,

    TimeTable,
    StatsTable,
    DistTable,
    SpeciesTable,

    TimePieChart,
    SpeciesPieChart,

    SpeciesSparkline,

    MetricDetail,

    Help,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RunningState {
    pub engine: bool,
    pub ui: bool,
    pub paused: bool,
}

pub struct SearchState {
    pub query: String,
    pub active: bool,
}

pub struct DisplayState {
    pub show_tag_filters: bool,
    pub show_help: bool,
    pub chart_id: LineChartType,
    pub previous_tab: TabId,
    pub tab_id: TabId,
    pub focus_panel: PanelId,
    pub prev_focus_panel: PanelId,
    pub modal_panel: Option<PanelId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabId {
    Dashboard,
    MetricChart,
    SearchBar,
    Help,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DashboardTab {
    Stats,
    Time,
    Distribution,
    Species,
}

impl DashboardTab {
    pub fn next(self) -> Self {
        match self {
            DashboardTab::Stats => DashboardTab::Time,
            DashboardTab::Time => DashboardTab::Distribution,
            DashboardTab::Distribution => DashboardTab::Species,
            DashboardTab::Species => DashboardTab::Stats,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            DashboardTab::Stats => DashboardTab::Species,
            DashboardTab::Time => DashboardTab::Stats,
            DashboardTab::Distribution => DashboardTab::Time,
            DashboardTab::Species => DashboardTab::Distribution,
        }
    }
}

pub struct ObjectiveState {
    pub objective: Objective,
    pub charts_visible: usize,
    pub chart_start_index: usize,
    pub objective_index: usize,
}

pub struct AppState<C: Chromosome> {
    pub front: Option<Front<Phenotype<C>>>,
    pub last_render: Option<Instant>,
    pub render_interval: Duration,
    pub chart_state: ChartState,
    pub dashboard_tab: DashboardTab,

    pub running: RunningState,
    pub display: DisplayState,

    pub search_state: SearchState,

    pub objective_state: ObjectiveState,
    pub metrics: MetricSet,
    pub species: Option<Vec<SpeciesSnapshot>>,
    pub index: usize,
    pub score: Score,

    pub time_table: AppTableState<&'static str>,
    pub stats_table: AppTableState<&'static str>,
    pub dist_table: AppTableState<&'static str>,
    pub species_table: AppTableState<SpeciesId>,
}

impl<C: Chromosome> AppState<C> {
    pub fn start_search(&mut self) {
        if self.display.modal_panel.is_some() {
            return;
        }

        self.display.prev_focus_panel = self.display.focus_panel;
        self.display.previous_tab = self.display.tab_id;
        self.display.tab_id = TabId::SearchBar;
        self.display.focus_panel = PanelId::Search;
        self.search_state.active = true;
    }

    pub fn stop_search(&mut self) {
        self.display.focus_panel = self.display.prev_focus_panel;
        let prev_tab = self.display.previous_tab;
        self.display.previous_tab = self.display.tab_id;
        self.display.tab_id = prev_tab;
        self.search_state.active = false;
    }

    pub fn clear_search(&mut self) {
        self.search_state.query.clear();
    }

    pub fn push_search_char(&mut self, c: char) {
        self.search_state.query.push(c);
    }

    pub fn pop_search_char(&mut self) {
        self.search_state.query.pop();
    }

    pub fn metric_matches_search(&self, metric: &Metric) -> bool {
        let query = self.search_state.query.trim();
        if query.is_empty() {
            return true;
        }

        let q = query.to_lowercase();

        metric.name().to_lowercase().contains(&q)
            || metric
                .tags_iter()
                .any(|tag| tag.as_str().to_lowercase().contains(&q))
    }
}

impl<C: Chromosome> AppState<C> {
    pub fn render_interval(&self) -> Duration {
        self.render_interval
    }

    pub fn set_objective_index(&mut self, index: usize) {
        if index < self.objective_state.objective.dims() {
            self.objective_state.objective_index = index;
        }
    }

    pub fn get_panel_block(&self, panel: PanelId) -> Block<'static> {
        if self.display.focus_panel == panel {
            border_style::BorderStyle::Rounded
                .block()
                .border_style(crate::styles::BORDER_GREEN)
        } else {
            border_style::BorderStyle::Rounded.block()
        }
    }

    pub fn toggle_help(&mut self) {
        self.display.show_help = !self.display.show_help;
        if self.display.show_help {
            self.display.previous_tab = self.display.tab_id;
            self.display.tab_id = TabId::Help;
            self.display.modal_panel = Some(PanelId::Help);
        } else {
            let prev_tab = self.display.previous_tab;
            self.display.previous_tab = self.display.tab_id;
            self.display.tab_id = prev_tab;
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

        if !self.search_state.active {
            self.search_state.query.clear();
        }

        if !self.display.show_tag_filters {
            return;
        }
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
        chart_type: LineChartType,
    ) -> Option<&RollingLineChart> {
        self.chart_state.get_line_chart(key, chart_type)
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

    pub fn chart_state(&self) -> &ChartState {
        &self.chart_state
    }

    pub fn active_tab_index(&self, tab: &TabId) -> usize {
        match tab {
            TabId::Dashboard => match self.dashboard_tab {
                DashboardTab::Stats => 0,
                DashboardTab::Time => 1,
                DashboardTab::Distribution => 2,
                DashboardTab::Species => 3,
            },
            TabId::MetricChart => match self.display.chart_id {
                LineChartType::Value => 0,
                LineChartType::Mean => 1,
                LineChartType::Stddev => 2,
                LineChartType::Variance => 3,
            },
            TabId::SearchBar => 0,
            TabId::Help => 0,
        }
    }

    pub fn next_tab(&mut self) {
        match self.display.tab_id {
            TabId::Dashboard => {
                self.display.prev_focus_panel = self.display.focus_panel;
                self.dashboard_tab = self.dashboard_tab.next();

                self.display.focus_panel = match self.dashboard_tab {
                    DashboardTab::Time => PanelId::TimeTable,
                    DashboardTab::Stats => PanelId::StatsTable,
                    DashboardTab::Distribution => PanelId::DistTable,
                    DashboardTab::Species => PanelId::SpeciesTable,
                };
            }
            TabId::MetricChart => {
                self.display.chart_id = self.display.chart_id.next();
            }
            _ => return,
        }
    }

    pub fn previous_tab(&mut self) {
        match self.display.tab_id {
            TabId::Dashboard => {
                self.display.prev_focus_panel = self.display.focus_panel;
                self.dashboard_tab = self.dashboard_tab.previous();

                self.display.focus_panel = match self.dashboard_tab {
                    DashboardTab::Time => PanelId::TimeTable,
                    DashboardTab::Stats => PanelId::StatsTable,
                    DashboardTab::Distribution => PanelId::DistTable,
                    DashboardTab::Species => PanelId::SpeciesTable,
                };
            }
            TabId::MetricChart => {
                self.display.chart_id = self.display.chart_id.previous();
            }
            _ => return,
        }
    }

    pub fn move_selection_down(&mut self) {
        if let Some(PanelId::MetricModal) = self.display.modal_panel {
            return;
        }

        match self.dashboard_tab {
            DashboardTab::Time => Self::update_selection_down(&mut self.time_table),
            DashboardTab::Stats => Self::update_selection_down(&mut self.stats_table),
            DashboardTab::Distribution => Self::update_selection_down(&mut self.dist_table),
            DashboardTab::Species => Self::update_selection_down(&mut self.species_table),
        };
    }

    fn update_selection_down<T>(table_state: &mut AppTableState<T>) {
        if table_state.row_count == 0 {
            return;
        }

        if let Some(i) = table_state.state.selected() {
            let next = if i + 1 >= table_state.row_count {
                0
            } else {
                i + 1
            };

            table_state.state.select(Some(next));
            table_state.selected_row = next;
            table_state.scroll_bar = table_state.scroll_bar.as_mut().map(|sb| sb.position(next));
        } else {
            table_state.state.select(Some(0));
            table_state.selected_row = 0;
            table_state.scroll_bar = table_state.scroll_bar.as_mut().map(|sb| sb.position(0));
        }
    }

    pub fn move_selection_up(&mut self) {
        if let Some(PanelId::MetricModal) = self.display.modal_panel {
            return;
        }

        match self.dashboard_tab {
            DashboardTab::Time => Self::update_selection_up(&mut self.time_table),
            DashboardTab::Stats => Self::update_selection_up(&mut self.stats_table),
            DashboardTab::Distribution => Self::update_selection_up(&mut self.dist_table),
            DashboardTab::Species => Self::update_selection_up(&mut self.species_table),
        }
    }

    fn update_selection_up<T>(table_state: &mut AppTableState<T>) {
        if table_state.row_count == 0 {
            return;
        }

        if let Some(i) = table_state.state.selected() {
            let next = if i == 0 {
                table_state.row_count - 1 // wrap to last
            } else {
                i - 1
            };

            table_state.state.select(Some(next));
            table_state.selected_row = next;
            table_state.scroll_bar = table_state.scroll_bar.as_mut().map(|sb| sb.position(next));
        } else {
            let last = table_state.row_count - 1;
            table_state.state.select(Some(last));
            table_state.selected_row = last;
            table_state.scroll_bar = table_state.scroll_bar.as_mut().map(|sb| sb.position(last));
        }
    }

    pub fn get_selected_metric(&self) -> Option<&'static str> {
        match self.dashboard_tab {
            DashboardTab::Time => self.time_table.selected_value,
            DashboardTab::Stats => self.stats_table.selected_value,
            DashboardTab::Distribution => self.dist_table.selected_value,
            DashboardTab::Species => None,
        }
    }

    pub fn view_metric(&mut self) {
        if matches!(
            self.display.focus_panel,
            PanelId::TimeTable | PanelId::StatsTable | PanelId::DistTable | PanelId::MetricModal
        ) {
            self.display.modal_panel = match self.display.modal_panel {
                Some(PanelId::MetricModal) => None,
                _ => Some(PanelId::MetricModal),
            };

            if self.display.modal_panel.is_some() {
                self.display.prev_focus_panel = self.display.focus_panel;
                self.display.focus_panel = PanelId::MetricModal;
                self.display.tab_id = TabId::MetricChart;
            } else {
                self.display.focus_panel = self.display.prev_focus_panel;
                self.display.tab_id = TabId::Dashboard;
            }

            return;
        }
    }

    pub fn update_metrics(&mut self, metrics: MetricSet) {
        for metric in metrics.iter() {
            self.chart_state.update_from_metric(metric.1);
        }

        self.metrics = metrics;
    }

    pub fn update_species(&mut self, species: Option<Vec<SpeciesSnapshot>>) {
        self.species = species;

        if let Some(species) = &mut self.species {
            species.sort_unstable_by(|a, b| a.id.0.cmp(&b.id.0));
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
            dashboard_tab: DashboardTab::Stats,

            running: RunningState {
                engine: false,
                ui: true,
                paused: false,
            },

            display: DisplayState {
                show_tag_filters: false,
                show_help: false,
                chart_id: LineChartType::Mean,
                tab_id: TabId::Dashboard,
                previous_tab: TabId::Dashboard,
                focus_panel: PanelId::StatsTable,
                prev_focus_panel: PanelId::StatsTable,
                modal_panel: None,
            },

            objective_state: ObjectiveState {
                objective: Objective::Single(Optimize::Maximize),
                charts_visible: 2,
                chart_start_index: 0,
                objective_index: 0,
            },

            search_state: SearchState {
                query: String::new(),
                active: false,
            },

            time_table: AppTableState::new(),
            stats_table: AppTableState::new(),
            dist_table: AppTableState::new(),
            species_table: AppTableState::new(),
            species: None,
            metrics: MetricSet::new(),
            index: 0,
            score: Score::default(),
        }
    }
}

pub struct AppTableState<T> {
    pub state: TableState,
    pub scroll_bar: Option<ScrollbarState>,
    pub selected_value: Option<T>,
    pub selected_row: usize,
    pub row_count: usize,
    pub prev_row_count: usize,
}

impl<T> AppTableState<T> {
    pub fn new() -> Self {
        Self {
            state: TableState::default(),
            scroll_bar: None,
            selected_row: 0,
            row_count: 0,
            prev_row_count: 0,
            selected_value: None,
        }
    }

    pub fn update_rows<K, F>(&mut self, items: &[K], func: F)
    where
        F: Fn(&K) -> T,
    {
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

        self.selected_value = items.get(self.selected_row).map(&func);
    }
}
