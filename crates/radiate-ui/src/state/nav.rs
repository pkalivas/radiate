use super::chart::MetricChartType;
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

    /// The focusable panes this tab lays out, in `Tab`-cycle order. Every tab
    /// currently has the same shape: a list, a chart, and a detail panel.
    pub fn panes(self) -> &'static [Pane] {
        &[Pane::List, Pane::Chart, Pane::Detail]
    }
}

/// A focusable region within the active dashboard tab.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    List,
    Chart,
    Detail,
}

pub struct SearchState {
    pub query: String,
    pub active: bool,
}

pub struct NavState {
    pub mode: UiMode,
    pub dashboard_tab: DashboardTab,
    pub focus: Pane,
    pub chart_tab: MetricChartType,
    pub search: SearchState,
}

impl NavState {
    pub fn is_pane_focused(&self, pane: Pane) -> bool {
        self.mode == UiMode::Dashboard && self.focus == pane
    }

    pub fn next_pane(&mut self) {
        if let UiMode::Dashboard = self.mode {
            self.focus = cycle(self.dashboard_tab.panes(), self.focus, 1);
        }
    }

    pub fn previous_pane(&mut self) {
        if let UiMode::Dashboard = self.mode {
            self.focus = cycle(self.dashboard_tab.panes(), self.focus, -1);
        }
    }

    fn clamp_focus(&mut self) {
        let panes = self.dashboard_tab.panes();
        if !panes.contains(&self.focus) {
            self.focus = panes[0];
        }
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
            self.clamp_focus();
        }
    }

    pub fn previous_tab(&mut self) {
        if let UiMode::Dashboard = self.mode {
            self.dashboard_tab = self.dashboard_tab.previous();
            self.clamp_focus();
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
            focus: Pane::List,
            chart_tab: MetricChartType::Mean,
            search: SearchState {
                query: String::new(),
                active: false,
            },
        }
    }
}

/// Step through `panes` from `current` by `dir` (±1), wrapping.
fn cycle(panes: &[Pane], current: Pane, dir: isize) -> Pane {
    let n = panes.len() as isize;
    let i = panes.iter().position(|p| *p == current).unwrap_or(0) as isize;
    panes[(((i + dir) % n + n) % n) as usize]
}
