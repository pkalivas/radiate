mod chart;
mod dashboard;
mod state;
mod store;
mod ui;

use crate::ui::EngineUi;
use radiate_engines::{Chromosome, GeneticEngine};
use std::time::Duration;

pub fn dashboard<C, T>(engine: GeneticEngine<C, T>) -> GeneticEngine<C, T>
where
    C: Chromosome + Clone,
    T: Clone + Send + Sync,
{
    let dash = EngineUi::new(Duration::from_millis(500));
    engine.subscribe(dash)
}
