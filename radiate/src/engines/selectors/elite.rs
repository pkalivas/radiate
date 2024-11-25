use crate::{Gene, Optimize, Population};

use super::Select;

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

impl<G: Gene<G, A>, A> Select<G, A> for EliteSelector {
    fn name(&self) -> &'static str {
        "Elite Selector"
    }

    fn select(
        &self,
        population: &Population<G, A>,
        _: &Optimize,
        count: usize,
    ) -> Population<G, A> {
        population
            .iter()
            .take(count).cloned()
            .collect()
    }
}
