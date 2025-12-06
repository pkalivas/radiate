// use radiate_engines::Objective;
// use ratatui::widgets::{ListState, TableState};
// use std::{collections::HashMap, hash::Hash};

// pub struct StatefulList<T> {
//     pub state: ListState,
//     pub items: Vec<T>,
// }

// impl<T> StatefulList<T> {
//     pub fn with_items(items: Vec<T>) -> Self {
//         Self {
//             state: ListState::default(),
//             items,
//         }
//     }

//     pub fn next(&mut self) {
//         let i = match self.state.selected() {
//             Some(i) => {
//                 if i >= self.items.len() - 1 {
//                     0
//                 } else {
//                     i + 1
//                 }
//             }
//             None => 0,
//         };
//         self.state.select(Some(i));
//     }

//     pub fn previous(&mut self) {
//         let i = match self.state.selected() {
//             Some(i) => {
//                 if i == 0 {
//                     self.items.len() - 1
//                 } else {
//                     i - 1
//                 }
//             }
//             None => 0,
//         };
//         self.state.select(Some(i));
//     }
// }

// pub(crate) enum StoreAction {
//     SetRunning(bool),
//     SetGeneration(usize),
//     SetObjective(Objective),
//     UpdateChart(&'static str, (f64, f64)),
// }

// #[derive(Default)]
// pub(crate) struct Store {
//     pub running: bool,
//     pub generation: usize,
//     pub objective: Option<Objective>,
//     pub charts: HashMap<&'static str, Vec<(f64, f64)>>,
//     pub tables: HashMap<&'static str, TableState>,
// }

// impl Store {
//     #[inline]
//     pub fn dispatch(&mut self, action: StoreAction) {
//         match action {
//             StoreAction::SetRunning(running) => {
//                 self.running = running;
//             }
//             StoreAction::SetGeneration(generation) => {
//                 self.generation = generation;
//             }
//             StoreAction::SetObjective(objective) => {
//                 self.objective = Some(objective);
//             }
//             StoreAction::UpdateChart(name, point) => {
//                 self.charts.entry(name).or_default().push(point);
//             }
//         }
//     }
// }
