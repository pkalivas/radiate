use super::Select;
use crate::objectives::Objective;
use crate::{Chromosome, Population};

pub struct EliteSelector;

impl EliteSelector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EliteSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Chromosome> Select<C> for EliteSelector {
    fn name(&self) -> &'static str {
        "Elite Selector"
    }

    fn select(&self, population: &Population<C>, _: &Objective, count: usize) -> Population<C> {
        population.iter().take(count).cloned().collect()
    }
}
