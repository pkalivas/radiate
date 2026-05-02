use super::PanelId;
use super::chart::LineChartType;
use radiate_engines::Metric;
use ratatui::widgets::Block;
use tui_piechart::border_style;

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

pub struct SearchState {
    pub query: String,
    pub active: bool,
}

pub struct NavState {
    pub display: DisplayState,
    pub search: SearchState,
    pub dashboard_tab: DashboardTab,
}

impl NavState {
    pub fn start_search(&mut self) {
        if self.display.modal_panel.is_some() {
            return;
        }
        self.display.prev_focus_panel = self.display.focus_panel;
        self.display.previous_tab = self.display.tab_id;
        self.display.tab_id = TabId::SearchBar;
        self.display.focus_panel = PanelId::Search;
        self.search.active = true;
    }

    pub fn stop_search(&mut self) {
        self.display.focus_panel = self.display.prev_focus_panel;
        let prev_tab = self.display.previous_tab;
        self.display.previous_tab = self.display.tab_id;
        self.display.tab_id = prev_tab;
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
            _ => {}
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
            _ => {}
        }
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

    pub fn get_panel_block(&self, panel: PanelId) -> Block<'static> {
        if self.display.focus_panel == panel {
            border_style::BorderStyle::Rounded
                .block()
                .border_style(crate::styles::BORDER_GREEN)
        } else {
            border_style::BorderStyle::Rounded.block()
        }
    }

    pub fn clear_filters(&mut self) {
        if self.display.show_help {
            self.display.show_help = false;
            self.display.modal_panel = None;
        }

        if !self.search.active {
            self.search.query.clear();
        }

        if !self.display.show_tag_filters {
            return;
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
            search: SearchState {
                query: String::new(),
                active: false,
            },
            dashboard_tab: DashboardTab::Stats,
        }
    }
}
