use super::PanelId;
use super::chart::LineChartType;
use radiate_engines::Metric;
use ratatui::widgets::Block;
use tui_piechart::border_style;

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
}

pub struct SearchState {
    pub query: String,
    pub active: bool,
}

pub struct NavState {
    pub mode: UiMode,
    pub dashboard_tab: DashboardTab,
    pub chart_tab: LineChartType,
    pub focus_panel: PanelId,
    pub search: SearchState,
}

impl NavState {
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
            UiMode::Dashboard
                if matches!(
                    self.focus_panel,
                    PanelId::TimeTable | PanelId::StatsTable | PanelId::DistTable
                ) =>
            {
                self.mode = UiMode::MetricModal;
            }
            UiMode::MetricModal => self.mode = UiMode::Dashboard,
            _ => {}
        }
    }

    pub fn next_tab(&mut self) {
        match self.mode {
            UiMode::Dashboard => {
                self.dashboard_tab = self.dashboard_tab.next();
                self.focus_panel = match self.dashboard_tab {
                    DashboardTab::Time => PanelId::TimeTable,
                    DashboardTab::Stats => PanelId::StatsTable,
                    DashboardTab::Distribution => PanelId::DistTable,
                    DashboardTab::Species => PanelId::SpeciesTable,
                };
            }
            UiMode::MetricModal => self.chart_tab = self.chart_tab.next(),
            _ => {}
        }
    }

    pub fn previous_tab(&mut self) {
        match self.mode {
            UiMode::Dashboard => {
                self.dashboard_tab = self.dashboard_tab.previous();
                self.focus_panel = match self.dashboard_tab {
                    DashboardTab::Time => PanelId::TimeTable,
                    DashboardTab::Stats => PanelId::StatsTable,
                    DashboardTab::Distribution => PanelId::DistTable,
                    DashboardTab::Species => PanelId::SpeciesTable,
                };
            }
            UiMode::MetricModal => self.chart_tab = self.chart_tab.previous(),
            _ => {}
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

    pub fn chart_tab_index(&self) -> usize {
        match self.chart_tab {
            LineChartType::Value => 0,
            LineChartType::Mean => 1,
            LineChartType::Stddev => 2,
            LineChartType::Variance => 3,
        }
    }

    pub fn get_panel_block(&self, panel: PanelId) -> Block<'static> {
        let effective = match self.mode {
            UiMode::MetricModal => PanelId::MetricModal,
            UiMode::Search => PanelId::Search,
            _ => self.focus_panel,
        };
        if effective == panel {
            border_style::BorderStyle::Rounded
                .block()
                .border_style(crate::styles::BORDER_GREEN)
        } else {
            border_style::BorderStyle::Rounded.block()
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
            focus_panel: PanelId::StatsTable,
            search: SearchState {
                query: String::new(),
                active: false,
            },
        }
    }
}
