use super::Select;
use crate::objectives::Objective;
use crate::{Chromosome, EngineCompoment, Population};

pub struct EliteSelector;

impl EliteSelector {
    pub fn new() -> Self {
        EliteSelector
    }
}

impl Default for EliteSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineCompoment for EliteSelector {
    fn name(&self) -> &'static str {
        "EliteSelector"
    }
}

impl<C: Chromosome> Select<C> for EliteSelector {
    fn select(&self, population: &Population<C>, _: &Objective, count: usize) -> Population<C> {
        population.iter().take(count).cloned().collect()
    }
}
