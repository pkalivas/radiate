use crate::state::{AppState, DashboardTab};
use radiate_engines::{Chromosome, Metric, stats::TagType};

impl<C: Chromosome> AppState<C> {
    // pub fn select<O>(&self, input: impl Fn(&Self) -> O) -> O {
    //     input(self)
    // }

    pub fn get_current_metric(&self) -> Option<(&'static str, &Metric)> {
        let maybe_name = match self.dashboard_tab {
            DashboardTab::Time => self.time_table.selected_value,
            DashboardTab::Stats => self.stats_table.selected_value,
            DashboardTab::Distribution => self.dist_table.selected_value,
            DashboardTab::Species => None,
        };

        maybe_name.and_then(|name| self.metrics.get(name).map(|m| (name, m)))
    }
}

// pub trait Selector {
//     fn select<F, O>(&self, input: F) -> O
//     where
//         F: for<'a> Fn(&'a Self) -> O;
// }

// impl<C: Chromosome> Selector for AppState<C> {
//     fn select<F, O>(&self, input: F) -> O
//     where
//         F: Fn(&Self) -> O,
//     {
//         input(self)
//     }
// }

// pub fn select_current_metric<C: Chromosome>(
//     state: &AppState<C>,
// ) -> Option<(&'static str, &Metric)> {
//     let maybe_name = match state.dashboard_tab {
//         DashboardTab::Time => state.time_table.selected_value,
//         DashboardTab::Stats => state.stats_table.selected_value,
//         DashboardTab::Distribution => state.dist_table.selected_value,
//         DashboardTab::Species => None,
//     };

//     maybe_name.and_then(|name| Some((name, state.metrics.get(name)?)))
// }

// pub fn select_tagged_metrics<C: Chromosome>(
//     tag: TagType,
// ) -> impl Fn(&AppState<C>) -> Vec<(&'static str, &Metric)> {
//     move |state: &AppState<C>| {
//         let mut items = state
//             .metrics
//             .iter_tagged(tag)
//             .filter(|(_, m)| state.metric_has_tags(m))
//             .filter(|(_, m)| state.metric_matches_search(m))
//             .collect::<Vec<_>>();

//         items.sort_unstable_by(|a, b| a.0.cmp(b.0));
//         items
//     }
// }

// pub fn select_time_pie_bars<C: Chromosome>(state: &AppState<C>) -> Option<(&'static str, &Metric)> {
//     let name = state.time_table.selected_value?;
//     Some((name, state.metrics.get(name)?))
// }
