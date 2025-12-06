mod chart;
mod dashboard;
mod state;
mod ui;

use crate::ui::EngineUi;
use radiate_engines::{Chromosome, GeneticEngine};
use std::time::Duration;

pub fn dashboard<C, T>(engine: GeneticEngine<C, T>) -> EngineUi<C, T>
where
    C: Chromosome + Clone,
    T: Clone + Send + Sync,
{
    EngineUi::new(engine, Duration::from_millis(100))
}
