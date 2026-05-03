pub mod chart;
pub mod evo;
pub mod nav;
pub mod run;
pub mod tables;

use radiate_engines::{Chromosome, Metric};

pub use chart::LineChartType;
pub use evo::EvoState;
pub use nav::{DashboardTab, NavState, UiMode};
pub use run::RunState;
pub use tables::{AppTableState, TableStates};

pub struct AppState<C: Chromosome> {
    pub run: RunState,
    pub nav: NavState,
    pub evo: EvoState<C>,
    pub tables: TableStates,
}

impl<C: Chromosome> AppState<C> {
    pub fn move_selection_down(&mut self) {
        if matches!(self.nav.mode, UiMode::MetricModal) {
            return;
        }
        self.tables.move_down(self.nav.dashboard_tab);
    }

    pub fn move_selection_up(&mut self) {
        if matches!(self.nav.mode, UiMode::MetricModal) {
            return;
        }
        self.tables.move_up(self.nav.dashboard_tab);
    }

    pub fn get_selected_metric(&self) -> Option<&'static str> {
        self.tables.selected_metric(self.nav.dashboard_tab)
    }

    pub fn metric_matches_search(&self, metric: &Metric) -> bool {
        self.nav.metric_matches_search(metric)
    }
}

impl<C: Chromosome> Default for AppState<C> {
    fn default() -> Self {
        Self {
            run: RunState::default(),
            nav: NavState::default(),
            evo: EvoState::default(),
            tables: TableStates::default(),
        }
    }
}
