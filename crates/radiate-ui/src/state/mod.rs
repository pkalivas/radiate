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

/// Which way a cycling action goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    Next,
    Prev,
}

/// A movement through a list or a scrolled body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    Up,
    Down,
    PageUp,
    PageDown,
    Top,
    Bottom,
}

pub struct AppState<C: Chromosome> {
    pub run: RunState,
    pub nav: NavState,
    pub evo: EvoState<C>,
    pub tables: TableStates,
    pub inspect: InspectState,
    pub stream: StreamState,
}

impl<C: Chromosome> AppState<C> {
    /// Move the cursor in the active tab's list.
    pub fn move_selection(&mut self, motion: Motion) {
        self.tables
            .selection_mut(self.nav.dashboard_tab)
            .apply(motion);
    }

    pub fn get_selected_metric(&self) -> Option<&str> {
        self.tables.selected_metric(self.nav.dashboard_tab)
    }

    pub fn get_selected_phenotype(&self) -> Option<&Phenotype<C>> {
        if self.nav.dashboard_tab != DashboardTab::Population {
            return None;
        }

        let index = self.tables.population.selection.selected_row;
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

    /// Tab / Shift-Tab: step through the selected phenotype's chromosomes on
    /// Population, and through the selected metric's chart views elsewhere.
    pub fn cycle(&mut self, dir: Dir) {
        if self.nav.dashboard_tab == DashboardTab::Population {
            let count = self.selected_chromosome_count();
            match dir {
                Dir::Next => self.inspect.next_chromosome(count),
                Dir::Prev => self.inspect.previous_chromosome(count),
            }
            return;
        }

        let views = self.selected_metric_views();
        let n = views.len();
        let i = match dir {
            Dir::Next => (self.chart_view_index() + 1) % n,
            Dir::Prev => (self.chart_view_index() + n - 1) % n,
        };
        self.nav.set_chart_tab(views[i]);
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
