use super::{Dir, EvoState, chart::MetricChartType};
use crate::state::evo::EventLogEntry;
use radiate_engines::Chromosome;
use radiate_engines::Metric;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiMode {
    Dashboard,
    MetricModal,
    Search,
    Help,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DashboardTab {
    Stats,
    Time,
    Distribution,
    Events,
    Population,
    Species,
    Log,
    Front,
}

impl DashboardTab {
    /// Every tab, in the order the tab bar shows them. This is the only place
    /// the order is written down.
    pub const ALL: [DashboardTab; 8] = [
        DashboardTab::Stats,
        DashboardTab::Time,
        DashboardTab::Distribution,
        DashboardTab::Events,
        DashboardTab::Population,
        DashboardTab::Species,
        DashboardTab::Log,
        DashboardTab::Front,
    ];

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&tab| tab == self).unwrap_or(0)
    }

    pub fn title(self) -> &'static str {
        match self {
            DashboardTab::Stats => "Stats",
            DashboardTab::Time => "Time",
            DashboardTab::Distribution => "Distribution",
            DashboardTab::Events => "Events",
            DashboardTab::Population => "Population",
            DashboardTab::Species => "Species",
            DashboardTab::Log => "Improvements",
            DashboardTab::Front => "Front",
        }
    }

    /// Whether the run has anything to show on this tab. Stats is always
    /// available, which is what lets [`NavState::step_tab`] terminate.
    pub fn available<C: Chromosome>(self, evo: &EvoState<C>) -> bool {
        match self {
            DashboardTab::Species => evo.has_species(),
            DashboardTab::Log => !evo.is_multi(),
            DashboardTab::Front => evo.is_multi(),
            _ => true,
        }
    }

    fn step(self, dir: Dir) -> Self {
        let n = Self::ALL.len();
        let i = match dir {
            Dir::Next => (self.index() + 1) % n,
            Dir::Prev => (self.index() + n - 1) % n,
        };
        Self::ALL[i]
    }

    pub fn supports_metric_modal(self) -> bool {
        !matches!(
            self,
            DashboardTab::Species
                | DashboardTab::Log
                | DashboardTab::Front
                | DashboardTab::Events
                | DashboardTab::Population
        )
    }

    /// The focusable panes this tab lays out, in `Tab`-cycle order. Every tab
    /// currently has the same shape: a list, a chart, and a detail panel.
    /// Events is a single full-width list with no chart pane.
    pub fn panes(self) -> &'static [Pane] {
        match self {
            DashboardTab::Events => &[Pane::List],
            _ => &[Pane::List, Pane::Chart],
        }
    }
}

/// A focusable region within the active dashboard tab.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    List,
    Chart,
}

pub struct SearchState {
    pub query: String,
    pub active: bool,
}

pub struct NavState {
    pub mode: UiMode,
    pub dashboard_tab: DashboardTab,
    pub focus: Pane,
    pub search: SearchState,
    chart_tabs: [MetricChartType; DashboardTab::ALL.len()],
}

impl NavState {
    pub fn is_pane_focused(&self, pane: Pane) -> bool {
        self.focus == pane && matches!(self.mode, UiMode::Dashboard | UiMode::MetricModal)
    }

    /// The chart view remembered for the active dashboard tab.
    pub fn chart_tab(&self) -> MetricChartType {
        self.chart_tabs[self.dashboard_tab.index()]
    }

    /// Remember `view` for the active dashboard tab.
    pub fn set_chart_tab(&mut self, view: MetricChartType) {
        self.chart_tabs[self.dashboard_tab.index()] = view;
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
                self.focus = Pane::Chart;
            }
            UiMode::MetricModal => {
                self.mode = UiMode::Dashboard;
                self.focus = Pane::List;
            }
            _ => {}
        }
    }

    /// Move to the next available tab in `dir`.
    pub fn step_tab<C: Chromosome>(&mut self, dir: Dir, evo: &EvoState<C>) {
        let mut tab = self.dashboard_tab.step(dir);
        while !tab.available(evo) {
            tab = tab.step(dir);
        }
        self.dashboard_tab = tab;
        self.clamp_focus();
    }

    pub fn ensure_tab_available<C: Chromosome>(&mut self, evo: &EvoState<C>) {
        if !self.dashboard_tab.available(evo) {
            self.dashboard_tab = DashboardTab::Stats;
            self.clamp_focus();
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
                .iter_tags()
                .any(|tag| tag.as_str().to_lowercase().contains(&q))
    }

    pub fn log_matches_search(&self, log: &EventLogEntry) -> bool {
        let query = self.search.query.trim();
        if query.is_empty() {
            return true;
        }
        let q = query.to_lowercase();
        log.message.to_lowercase().contains(&q)
            || format!("{:?}", log.level).to_lowercase().contains(&q)
    }
}

impl Default for NavState {
    fn default() -> Self {
        Self {
            mode: UiMode::Dashboard,
            dashboard_tab: DashboardTab::Stats,
            focus: Pane::List,
            chart_tabs: DashboardTab::ALL.map(|tab| match tab {
                DashboardTab::Distribution => MetricChartType::BoxWhisker,
                _ => MetricChartType::Mean,
            }),
            search: SearchState {
                query: String::new(),
                active: false,
            },
        }
    }
}

// /// Step through `panes` from `current` by `dir` (±1), wrapping.
// fn cycle(panes: &[Pane], current: Pane, dir: isize) -> Pane {
//     let n = panes.len() as isize;
//     let i = panes.iter().position(|p| *p == current).unwrap_or(0) as isize;
//     panes[(((i + dir) % n + n) % n) as usize]
// }
