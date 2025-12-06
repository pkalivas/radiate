use crate::chart::ChartData;
use radiate_engines::{Metric, MetricSet, Objective, Optimize, Score, metric_names, stats::Tag};
use ratatui::widgets::TableState;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum MetricsTab {
    #[default]
    Time,
    Stats,
    Distributions,
}

pub struct ChartState {
    fitness: ChartData,
    keyed: HashMap<&'static str, ChartData>,
}

impl ChartState {
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

impl Default for ChartState {
    fn default() -> Self {
        Self {
            fitness: ChartData::new()
                .with_metric_name(metric_names::SCORES)
                .with_name("Score"),
            keyed: HashMap::new(),
        }
    }
}

pub(crate) struct AppState {
    pub last_render: Option<Instant>,
    pub render_interval: Duration,
    pub chart_state: ChartState,
    pub metrics_tab: MetricsTab,
    pub tag_view: Vec<usize>,
    pub all_tags: Vec<Tag>,

    pub display_tag_filters: bool,
    pub display_metric_charts: bool,
    pub display_metric_means: bool,
    pub is_running: bool,

    pub objective: Objective,
    pub metrics: MetricSet,
    pub index: usize,
    pub score: Score,

    pub render_count: usize,

    pub time_table: TableState,
    pub stats_table: TableState,
    pub distribution_table: TableState,
    pub time_row_count: usize,
    pub stats_row_count: usize,
    pub distribution_row_count: usize,
}

impl AppState {
    pub fn last_render(&self) -> Option<Instant> {
        self.last_render
    }

    pub fn render_interval(&self) -> Duration {
        self.render_interval
    }

    pub fn set_last_render(&mut self, instant: Option<Instant>) {
        self.last_render = instant;
    }

    pub fn toggle_display_tag_filters(&mut self) {
        self.display_tag_filters = !self.display_tag_filters;
    }

    pub fn toggle_display_metric_charts(&mut self) {
        self.display_metric_charts = !self.display_metric_charts;
    }

    pub fn toggle_display_metric_means(&mut self) {
        self.display_metric_means = !self.display_metric_means;
    }

    pub fn toggle_is_running(&mut self) {
        self.is_running = !self.is_running;
    }

    pub fn set_tag_filter_by_index(&mut self, index: usize) {
        if !self.display_tag_filters {
            return;
        }

        if self.tag_view.contains(&index) {
            self.tag_view.retain(|&i| i != index);
        } else {
            if index < self.all_tags.len() {
                self.tag_view.push(index);
            } else {
                self.tag_view.retain(|&i| i != index);
            }
        }
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

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn charts(&self) -> &ChartState {
        &self.chart_state
    }

    pub fn charts_mut(&mut self) -> &mut ChartState {
        &mut self.chart_state
    }

    pub fn metric_has_tags(&self, metric: &Metric) -> bool {
        if self.tag_view.is_empty() {
            true
        } else {
            for &tag_index in &self.tag_view {
                if let Some(tag) = self.all_tags.get(tag_index) {
                    if metric.contains_tag(tag) {
                        return true;
                    }
                }
            }

            false
        }
    }

    pub fn move_selection_down(&mut self) {
        let (state, max) = match self.metrics_tab {
            MetricsTab::Time => (&mut self.time_table, self.time_row_count),
            MetricsTab::Stats => (&mut self.stats_table, self.stats_row_count),
            MetricsTab::Distributions => {
                (&mut self.distribution_table, self.distribution_row_count)
            }
        };

        if max == 0 {
            return;
        }

        let i = match state.selected() {
            Some(i) if i + 1 < max => i + 1,
            _ => 0, // wrap to first
        };

        state.select(Some(i));
    }

    pub fn move_selection_up(&mut self) {
        let (state, max) = match self.metrics_tab {
            MetricsTab::Time => (&mut self.time_table, self.time_row_count),
            MetricsTab::Stats => (&mut self.stats_table, self.stats_row_count),
            MetricsTab::Distributions => {
                (&mut self.distribution_table, self.distribution_row_count)
            }
        };

        if max == 0 {
            return;
        }

        let i = match state.selected() {
            Some(0) => max - 1, // wrap to last
            Some(i) => i - 1,
            None => 0,
        };

        state.select(Some(i));
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            last_render: None,
            render_interval: Duration::from_millis(500),
            chart_state: ChartState::default(),
            metrics_tab: MetricsTab::Time,
            tag_view: Vec::new(),
            all_tags: Vec::new(),
            display_tag_filters: false,
            display_metric_charts: true,
            display_metric_means: false,
            is_running: false,
            time_table: TableState::default(),
            stats_table: TableState::default(),
            distribution_table: TableState::default(),
            objective: Objective::Single(Optimize::Minimize),
            metrics: MetricSet::new(),
            index: 0,
            score: Score::default(),
            time_row_count: 0,
            stats_row_count: 0,
            distribution_row_count: 0,
            render_count: 0,
        }
    }
}

// use std::collections::{HashMap, HashSet};

// impl DashboardState {
//     /// Map metric name to a ChartData index, or just track names:
//     pub extra_metric_names: HashSet<String>,
//     // or: pub extra_metric_charts: HashMap<String, ChartData>;

//     pub fn toggle_selected_metric_plot(&mut self) {
//         // determine which table is active and its selected index
//         let (selected_idx, list): (Option<usize>, Vec<&'static str>) = match self.metrics_tab {
//             MetricsTab::Time => {
//                 let idx = self.time_table.selected();
//                 let names: Vec<_> = self
//                     .metrics
//                     .iter_scope(MetricScope::Step)
//                     .map(|(name, _)| name)
//                     .chain(
//                         self.metrics
//                             .iter_tagged(metric_tags::SELECTOR)
//                             .map(|(name, _)| name),
//                     )
//                     .chain(
//                         self.metrics
//                             .iter_tagged(metric_tags::ALTERER)
//                             .map(|(name, _)| name),
//                     )
//                     .collect();
//                 (idx, names)
//             }
//             MetricsTab::Stats => {
//                 let idx = self.stats_table.selected();
//                 let mut items: Vec<_> = self.metrics.iter_scope(MetricScope::Generation).map(|(n,_)| n).collect();
//                 // you can add/remove tagged subsets here similarly
//                 (idx, items)
//             }
//             MetricsTab::Distributions => {
//                 let idx = self.distribution_table.selected();
//                 let items: Vec<_> = self
//                     .metrics
//                     .iter_tagged(metric_tags::DISTRIBUTION)
//                     .map(|(name, _)| name)
//                     .collect();
//                 (idx, items)
//             }
//         };

//         let Some(i) = selected_idx else { return; };
//         if i >= list.len() { return; }

//         let name = list[i].to_string();

//         if !self.extra_metric_names.insert(name.clone()) {
//             // was already there → toggle off
//             self.extra_metric_names.remove(&name);
//         }

//         // Updating the actual chart data can be done in `Dashboard::update`
//         // by checking extra_metric_names and pushing series accordingly.
//     }
// }
