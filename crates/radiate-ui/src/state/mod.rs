pub mod chart;
pub mod evo;
pub mod inspect;
pub mod nav;
pub mod run;
pub mod stream;
pub mod tables;

use radiate_engines::{Chromosome, Metric, Phenotype};

pub use chart::MetricChartType;
pub use evo::EvoState;
pub use inspect::InspectState;
pub use nav::{NavState, Pane, UiMode};
pub use run::RunState;
pub use stream::StreamState;
pub use tables::{AppTableState, TableStates};

use crate::state::nav::DashboardTab;

pub struct AppState<C: Chromosome> {
    pub run: RunState,
    pub nav: NavState,
    pub evo: EvoState<C>,
    pub tables: TableStates,
    pub inspect: InspectState,
    pub stream: StreamState,
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

    pub fn move_selection_page_down(&mut self) {
        if matches!(self.nav.mode, UiMode::MetricModal) {
            return;
        }
        self.tables.move_page_down(self.nav.dashboard_tab);
    }

    pub fn move_selection_page_up(&mut self) {
        if matches!(self.nav.mode, UiMode::MetricModal) {
            return;
        }
        self.tables.move_page_up(self.nav.dashboard_tab);
    }

    pub fn move_selection_to_top(&mut self) {
        if matches!(self.nav.mode, UiMode::MetricModal) {
            return;
        }
        self.tables.move_to_top(self.nav.dashboard_tab);
    }

    pub fn move_selection_to_bottom(&mut self) {
        if matches!(self.nav.mode, UiMode::MetricModal) {
            return;
        }
        self.tables.move_to_bottom(self.nav.dashboard_tab);
    }

    pub fn get_selected_metric(&self) -> Option<&str> {
        self.tables.selected_metric(self.nav.dashboard_tab)
    }

    pub fn get_selected_phenotype(&self) -> Option<&Phenotype<C>> {
        if self.nav.dashboard_tab != DashboardTab::Population {
            return None;
        }

        let index = self.tables.population.selected_row;
        self.evo
            .ecosystem
            .as_ref()
            .and_then(|eco| eco.get_phenotype(index))
    }

    pub fn selected_chromosome_count(&self) -> usize {
        self.get_selected_phenotype()
            .map(|p| p.genotype().len())
            .unwrap_or(0)
    }

    pub fn next_chromosome(&mut self) {
        let count = self.selected_chromosome_count();
        self.inspect.next_chromosome(count);
    }

    pub fn previous_chromosome(&mut self) {
        let count = self.selected_chromosome_count();
        self.inspect.previous_chromosome(count);
    }

    pub fn metric_matches_search(&self, metric: &Metric) -> bool {
        self.nav.metric_matches_search(metric)
    }

    pub fn selected_metric_views(&self) -> &'static [MetricChartType] {
        self.get_selected_metric()
            .and_then(|name| self.evo.metrics.get(name))
            .map(MetricChartType::for_metric)
            .unwrap_or(MetricChartType::SCALAR_VIEWS)
    }

    pub fn current_chart_view(&self) -> MetricChartType {
        let views = self.selected_metric_views();
        let stored = self.nav.chart_tab();
        if views.contains(&stored) {
            stored
        } else {
            views[0]
        }
    }

    pub fn chart_view_index(&self) -> usize {
        let current = self.current_chart_view();
        self.selected_metric_views()
            .iter()
            .position(|v| *v == current)
            .unwrap_or(0)
    }

    pub fn next_chart_view(&mut self) {
        let views = self.selected_metric_views();
        let next = (self.chart_view_index() + 1) % views.len();
        self.nav.set_chart_tab(views[next]);
    }

    pub fn prev_chart_view(&mut self) {
        let views = self.selected_metric_views();
        let prev = (self.chart_view_index() + views.len() - 1) % views.len();
        self.nav.set_chart_tab(views[prev]);
    }
}

impl<C: Chromosome> Default for AppState<C> {
    fn default() -> Self {
        Self {
            run: RunState::default(),
            nav: NavState::default(),
            evo: EvoState::default(),
            tables: TableStates::default(),
            inspect: InspectState::default(),
            stream: StreamState::default(),
        }
    }
}
