use super::chart::LineChartType;
use radiate_engines::Metric;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiMode {
    Dashboard,
    MetricModal,
    Search,
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

    pub fn supports_metric_modal(self) -> bool {
        !matches!(self, DashboardTab::Species)
    }
}

pub struct SearchState {
    pub query: String,
    pub active: bool,
}

pub struct NavState {
    pub mode: UiMode,
    pub dashboard_tab: DashboardTab,
    pub chart_tab: LineChartType,
    pub search: SearchState,
}

impl NavState {
    pub fn is_tab_focused(&self, tab: DashboardTab) -> bool {
        self.mode == UiMode::Dashboard && self.dashboard_tab == tab
    }

    pub fn is_search_focused(&self) -> bool {
        self.mode == UiMode::Search
    }

    pub fn open_search(&mut self) {
        if !matches!(self.mode, UiMode::Dashboard) {
            return;
        }
        self.mode = UiMode::Search;
        self.search.active = true;
    }

    pub fn close_search(&mut self) {
        self.mode = UiMode::Dashboard;
        self.search.active = false;
    }

    pub fn clear_search(&mut self) {
        self.search.query.clear();
    }

    pub fn push_search_char(&mut self, c: char) {
        self.search.query.push(c);
    }

    pub fn pop_search_char(&mut self) {
        self.search.query.pop();
    }

    pub fn toggle_help(&mut self) {
        match self.mode {
            UiMode::Help => self.mode = UiMode::Dashboard,
            _ => self.mode = UiMode::Help,
        }
    }

    pub fn toggle_metric_modal(&mut self) {
        match self.mode {
            UiMode::Dashboard if self.dashboard_tab.supports_metric_modal() => {
                self.mode = UiMode::MetricModal;
            }
            UiMode::MetricModal => self.mode = UiMode::Dashboard,
            _ => {}
        }
    }

    pub fn next_tab(&mut self) {
        if let UiMode::Dashboard = self.mode {
            self.dashboard_tab = self.dashboard_tab.next();
        }
    }

    pub fn previous_tab(&mut self) {
        if let UiMode::Dashboard = self.mode {
            self.dashboard_tab = self.dashboard_tab.previous();
        }
    }

    pub fn dashboard_tab_index(&self) -> usize {
        match self.dashboard_tab {
            DashboardTab::Stats => 0,
            DashboardTab::Time => 1,
            DashboardTab::Distribution => 2,
            DashboardTab::Species => 3,
        }
    }

    pub fn clear_search_query(&mut self) {
        if !self.search.active {
            self.search.query.clear();
        }
    }

    pub fn metric_matches_search(&self, metric: &Metric) -> bool {
        let query = self.search.query.trim();
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

impl Default for NavState {
    fn default() -> Self {
        Self {
            mode: UiMode::Dashboard,
            dashboard_tab: DashboardTab::Stats,
            chart_tab: LineChartType::Mean,
            search: SearchState {
                query: String::new(),
                active: false,
            },
        }
    }
}
