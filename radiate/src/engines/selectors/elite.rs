use crate::{Gene, Optimize, Population};

use super::Select;

pub struct EliteSelector;

impl EliteSelector {
    pub fn new() -> Self {
        Self
    }
}

impl<G: Gene<G, A>, A> Select<G, A> for EliteSelector {
    fn select(
        &self,
        population: &Population<G, A>,
        _: &Optimize,
        count: usize,
    ) -> Population<G, A> {
        population
            .iter()
            .take(count)
            .map(|phenotype| phenotype.clone())
            .collect()
    }
}
