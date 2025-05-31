use super::{AlterRegistry, DiversityRegistry, selector::SelectorRegistry};
use crate::{PyEngineBuilder, PyGeneType};
use radiate::{Chromosome, Epoch, GeneticEngineBuilder};

pub trait ComponentRegistry {
    fn apply<C, T, E>(
        &self,
        engine_builder: GeneticEngineBuilder<C, T, E>,
        py_builder: &PyEngineBuilder,
        gene_type: PyGeneType,
    ) -> GeneticEngineBuilder<C, T, E>
    where
        C: Chromosome + Clone + PartialEq + 'static,
        T: Clone + Send + 'static,
        E: Epoch<Chromosome = C> + 'static;
}

pub struct EngineRegistry {
    pub alters: AlterRegistry,
    pub diversity: DiversityRegistry,
    pub selectors: SelectorRegistry,
}

impl EngineRegistry {
    pub fn new() -> Self {
        EngineRegistry {
            alters: AlterRegistry::new(),
            diversity: DiversityRegistry::new(),
            selectors: SelectorRegistry::new(),
        }
    }
}

impl ComponentRegistry for EngineRegistry {
    fn apply<C, T, E>(
        &self,
        engine_builder: GeneticEngineBuilder<C, T, E>,
        py_builder: &PyEngineBuilder,
        gene_type: PyGeneType,
    ) -> GeneticEngineBuilder<C, T, E>
    where
        C: Chromosome + Clone + PartialEq + 'static,
        T: Clone + Send + 'static,
        E: Epoch<Chromosome = C> + 'static,
    {
        let engine_builder = self.alters.apply(engine_builder, py_builder, gene_type);
        let engine_builder = self.diversity.apply(engine_builder, py_builder, gene_type);
        let engine_builder = self.selectors.apply(engine_builder, py_builder, gene_type);

        engine_builder
    }
}
