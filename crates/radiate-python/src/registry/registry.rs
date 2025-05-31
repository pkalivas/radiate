use super::{AlterRegistry, DiversityRegistry, EvaluatorRegistry, selector::SelectorRegistry};
use crate::{ObjectValue, PyEngineBuilder, PyGeneType, events::PyEventHandler};
use radiate::{Chromosome, Epoch, GeneticEngineBuilder};

pub trait ComponentRegistry {
    fn apply<C, E>(
        &self,
        engine_builder: GeneticEngineBuilder<C, ObjectValue, E>,
        py_builder: &PyEngineBuilder,
        gene_type: PyGeneType,
    ) -> GeneticEngineBuilder<C, ObjectValue, E>
    where
        C: Chromosome + Clone + PartialEq + 'static,
        E: Epoch<Chromosome = C> + 'static;
}

pub struct EngineRegistry {
    pub alters: AlterRegistry,
    pub diversity: DiversityRegistry,
    pub selectors: SelectorRegistry,
    pub evaluators: EvaluatorRegistry,
    pub handlers: EventHandlerRegistry,
}

impl EngineRegistry {
    pub fn new() -> Self {
        EngineRegistry {
            alters: AlterRegistry::new(),
            diversity: DiversityRegistry::new(),
            selectors: SelectorRegistry::new(),
            evaluators: EvaluatorRegistry::new(),
            handlers: EventHandlerRegistry::new(),
        }
    }
}

impl ComponentRegistry for EngineRegistry {
    fn apply<C, E>(
        &self,
        engine_builder: GeneticEngineBuilder<C, ObjectValue, E>,
        py_builder: &PyEngineBuilder,
        gene_type: PyGeneType,
    ) -> GeneticEngineBuilder<C, ObjectValue, E>
    where
        C: Chromosome + Clone + PartialEq + 'static,
        E: Epoch<Chromosome = C> + 'static,
    {
        let engine_builder = self.alters.apply(engine_builder, py_builder, gene_type);
        let engine_builder = self.diversity.apply(engine_builder, py_builder, gene_type);
        let engine_builder = self.selectors.apply(engine_builder, py_builder, gene_type);
        let engine_builder = self.evaluators.apply(engine_builder, py_builder, gene_type);
        let engine_builder = self.handlers.apply(engine_builder, py_builder, gene_type);

        engine_builder
    }
}

pub struct EventHandlerRegistry;

impl EventHandlerRegistry {
    pub fn new() -> Self {
        EventHandlerRegistry
    }
}

impl ComponentRegistry for EventHandlerRegistry {
    fn apply<C, E>(
        &self,
        engine_builder: GeneticEngineBuilder<C, ObjectValue, E>,
        py_builder: &PyEngineBuilder,
        _: PyGeneType,
    ) -> GeneticEngineBuilder<C, ObjectValue, E>
    where
        C: Chromosome + Clone + PartialEq + 'static,
        E: Epoch<Chromosome = C> + 'static,
    {
        if let Some(handlers) = py_builder.event_handlers.as_ref() {
            let mut builder = engine_builder;
            for handler in handlers {
                builder = builder.register(PyEventHandler::new(handler.clone()));
            }
            builder
        } else {
            engine_builder
        }
    }
}
