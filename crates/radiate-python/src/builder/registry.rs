use super::{
    AlterConfig, AlterType, DiversityConfig, DiversityType, ParamMapper, alter_name_to_type,
    diversity_string_to_type,
    selector::{
        BoltzmannSelectorMapper, EliteSelectorMapper, LinearRankSelectorMapper,
        NSGA2SelectorMapper, RankSelectorMapper, RouletteSelectorMapper, SelectorConfig,
        SelectorType, StochasticUniversalSamplingSelectorMapper, TournamentSelectorMapper,
        selector_string_to_type,
    },
};
use crate::{PyEngineBuilder, PyEngineParam, PyGeneType};
use radiate::{
    Alter, BitChromosome, CharChromosome, Chromosome, Diversity, Epoch, FloatChromosome,
    GeneticEngineBuilder, IntChromosome, Select,
};
use std::{collections::HashMap, sync::Arc};

pub enum ComponentType {
    Alter,
    Diversity,
    SurvivorSelector,
    OffspringSelector,
}

pub struct EngineComponent<C: Chromosome, T> {
    component_type: ComponentType,
    mapper: Box<dyn ParamMapper<C, Output = T>>,
}

impl<C: Chromosome, T> EngineComponent<C, T> {
    pub fn new(
        component_type: ComponentType,
        mapper: impl ParamMapper<C, Output = T> + 'static,
    ) -> Self {
        EngineComponent {
            component_type,
            mapper: Box::new(mapper),
        }
    }
}

pub struct EngineRegistry {
    pub alters: HashMap<PyGeneType, HashMap<AlterType, AlterConfig>>,
    pub diversity: HashMap<PyGeneType, HashMap<DiversityType, DiversityConfig>>,
    pub selectors: HashMap<SelectorType, SelectorConfig>,
}

impl EngineRegistry {
    pub fn new() -> Self {
        let mut registry = EngineRegistry {
            alters: HashMap::new(),
            diversity: HashMap::new(),
            selectors: HashMap::new(),
        };

        Self::register_float_alters(&mut registry);
        Self::register_int_alters(&mut registry);
        Self::register_char_alters(&mut registry);
        Self::register_bit_alters(&mut registry);

        Self::register_float_diversity_mappers(&mut registry);
        Self::register_int_diversity_mappers(&mut registry);
        Self::register_char_diversity_mappers(&mut registry);
        Self::register_bit_diversity_mappers(&mut registry);

        registry
    }

    pub fn set_engine_alters<C, T, E>(
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
        let mut result = Vec::new();
        let alters_map = self.alters.get(&gene_type);

        if let Some(alters) = alters_map {
            for param in py_builder.alters.iter() {
                let alter_type = alter_name_to_type(param.name());
                if let Some(alter_config) = alters.get(&alter_type) {
                    result.extend(match alter_config {
                        AlterConfig::Float(mapper) => mapper(param)
                            .into_iter()
                            .map(|alter| Self::transmute_alter::<FloatChromosome, C>(alter))
                            .collect::<Vec<_>>(),
                        AlterConfig::Int(mapper) => mapper(param)
                            .into_iter()
                            .map(|alter| Self::transmute_alter::<IntChromosome<i32>, C>(alter))
                            .collect::<Vec<_>>(),
                        AlterConfig::Char(mapper) => mapper(param)
                            .into_iter()
                            .map(|alter| Self::transmute_alter::<CharChromosome, C>(alter))
                            .collect::<Vec<_>>(),
                        AlterConfig::Bit(mapper) => mapper(param)
                            .into_iter()
                            .map(|alter| Self::transmute_alter::<BitChromosome, C>(alter))
                            .collect::<Vec<_>>(),
                    });
                }
            }
        }

        engine_builder.alter(result)
    }

    pub fn set_engine_diversity<C, T, E>(
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
        let diversity_type =
            diversity_string_to_type(py_builder.diversity.as_ref().unwrap().name())
                .expect("Invalid diversity type in parameter");
        if let Some(diversity_map) = self.diversity.get(&gene_type) {
            if let Some(diversity_config) = diversity_map.get(&diversity_type) {
                let diversity_param = match diversity_config {
                    DiversityConfig::Float(mapper) => {
                        Self::transmute_diversity::<FloatChromosome, C>(mapper(
                            py_builder.diversity.as_ref().unwrap(),
                        ))
                    }
                    DiversityConfig::Int(mapper) => {
                        Self::transmute_diversity::<IntChromosome<i32>, C>(mapper(
                            py_builder.diversity.as_ref().unwrap(),
                        ))
                    }
                    DiversityConfig::Char(mapper) => {
                        Self::transmute_diversity::<CharChromosome, C>(mapper(
                            py_builder.diversity.as_ref().unwrap(),
                        ))
                    }
                    DiversityConfig::Bit(mapper) => Self::transmute_diversity::<BitChromosome, C>(
                        mapper(py_builder.diversity.as_ref().unwrap()),
                    ),
                };

                return engine_builder.with_values(|params| {
                    params.diversity = Some(diversity_param);
                    params.species_threshold = py_builder.species_threshold;
                });
            }
        }

        engine_builder
    }

    pub fn set_engine_selectors<C, T, E>(
        &self,
        engine_builder: GeneticEngineBuilder<C, T, E>,
        py_builder: &PyEngineBuilder,
    ) -> GeneticEngineBuilder<C, T, E>
    where
        C: Chromosome + Clone + PartialEq + 'static,
        T: Clone + Send + 'static,
        E: Epoch<Chromosome = C> + 'static,
    {
        let survivor_selector_type = selector_string_to_type(py_builder.survivor_selector.name())
            .map(|surv| self.selectors.get(&surv))
            .flatten()
            .expect("Invalid survivor selector type in parameter");
        let offspring_selector_type = selector_string_to_type(py_builder.offspring_selector.name())
            .map(|offs| self.selectors.get(&offs))
            .flatten()
            .expect("Invalid offspring selector type in parameter");

        let offspring_selector = &py_builder.offspring_selector;

        todo!()
    }

    pub fn register_selector_mapper<C: Chromosome>(&mut self, selector_type: SelectorType) {
        let selector_config = match selector_type {
            SelectorType::Tournament => SelectorConfig::Tournament(TournamentSelectorMapper),
            SelectorType::Roulette => SelectorConfig::Roulette(RouletteSelectorMapper),
            SelectorType::Rank => SelectorConfig::Rank(RankSelectorMapper),
            SelectorType::Elitism => SelectorConfig::Elitism(EliteSelectorMapper),
            SelectorType::Boltzmann => SelectorConfig::Boltzmann(BoltzmannSelectorMapper),
            SelectorType::StochasticUniversalSampling => {
                SelectorConfig::StochasticUniversalSampling(
                    StochasticUniversalSamplingSelectorMapper,
                )
            }
            SelectorType::LinearRank => SelectorConfig::LinearRank(LinearRankSelectorMapper),
            SelectorType::NSGA2 => SelectorConfig::NSGA2(NSGA2SelectorMapper),
        };

        self.selectors.insert(selector_type, selector_config);
    }

    pub fn register_diversity_mapper<C, M>(
        &mut self,
        gene_type: PyGeneType,
        diversity_type: DiversityType,
        mapper: M,
    ) where
        C: Chromosome + 'static,
        M: ParamMapper<C, Output = Arc<dyn Diversity<C>>> + 'static,
    {
        let config = match gene_type {
            PyGeneType::Float => DiversityConfig::Float(Box::new(move |param| {
                Self::transmute_diversity(mapper.map(param))
            })),
            PyGeneType::Int => DiversityConfig::Int(Box::new(move |param| {
                Self::transmute_diversity(mapper.map(param))
            })),
            PyGeneType::Char => DiversityConfig::Char(Box::new(move |param| {
                Self::transmute_diversity(mapper.map(param))
            })),
            PyGeneType::Bit => DiversityConfig::Bit(Box::new(move |param| {
                Self::transmute_diversity(mapper.map(param))
            })),
            _ => panic!("Unsupported gene type"),
        };

        self.diversity
            .entry(gene_type)
            .or_insert_with(HashMap::new)
            .insert(diversity_type, config);
    }

    pub fn register_alter_mapper<
        C: Chromosome + 'static,
        M: ParamMapper<C, Output = Vec<Box<dyn Alter<C>>>> + 'static,
    >(
        &mut self,
        gene_type: PyGeneType,
        alter_type: AlterType,
        mapper: M,
    ) {
        let alter_config = match gene_type {
            PyGeneType::Float => AlterConfig::Float(Box::new(move |param| {
                mapper
                    .map(param)
                    .into_iter()
                    .map(|alter| Self::transmute_alter::<C, FloatChromosome>(alter))
                    .collect::<Vec<_>>()
            })),
            PyGeneType::Int => AlterConfig::Int(Box::new(move |param| {
                mapper
                    .map(param)
                    .into_iter()
                    .map(|alter| Self::transmute_alter::<C, IntChromosome<i32>>(alter))
                    .collect::<Vec<_>>()
            })),
            PyGeneType::Char => AlterConfig::Char(Box::new(move |param| {
                mapper
                    .map(param)
                    .into_iter()
                    .map(|alter| Self::transmute_alter::<C, CharChromosome>(alter))
                    .collect::<Vec<_>>()
            })),
            PyGeneType::Bit => AlterConfig::Bit(Box::new(move |param| {
                mapper
                    .map(param)
                    .into_iter()
                    .map(|alter| Self::transmute_alter::<C, BitChromosome>(alter))
                    .collect::<Vec<_>>()
            })),
            _ => panic!("Unsupported gene type"),
        };
        self.alters
            .entry(gene_type)
            .or_insert_with(HashMap::new)
            .entry(alter_type)
            .or_insert_with(|| alter_config);
    }

    fn transmute_diversity<C: Chromosome + 'static, T: Chromosome>(
        diversity: Arc<dyn Diversity<C>>,
    ) -> Arc<dyn Diversity<T>> {
        // SAFETY: This transmute is safe because we ensure that the Diversity<C> can be used as Diversity<T> during creation,
        // its not even possible to register the diversity if C and T are not compatible.
        unsafe { std::mem::transmute::<Arc<dyn Diversity<C>>, Arc<dyn Diversity<T>>>(diversity) }
    }

    fn transmute_alter<C: Chromosome + 'static, T: Chromosome>(
        alter: Box<dyn Alter<C>>,
    ) -> Box<dyn Alter<T>> {
        // SAFETY: This transmute is safe because we ensure that the Alter<C> can be used as Alter<T> during creation,
        // its not even possible to register the alter if C and T are not compatible.
        unsafe { std::mem::transmute::<Box<dyn Alter<C>>, Box<dyn Alter<T>>>(alter) }
    }
}

// let survivor_selector_type = selector_string_to_type(py_builder.survivor_selector.name())
//     .map(|surv| self.selectors.get(&surv))
//     .flatten()
//     .expect("Invalid survivor selector type in parameter");
// let offspring_selector_type = selector_string_to_type(py_builder.offspring_selector.name())
//     .map(|offs| self.selectors.get(&offs))
//     .flatten()
//     .expect("Invalid offspring selector type in parameter");

// let t = match survivor_selector_type {
//     SelectorConfig::Tournament(mapper) => engine_builder
//         .survivor_selector(ParamMapper::<C>::map(mapper, &py_builder.survivor_selector)),
//     SelectorConfig::Roulette(mapper) => engine_builder
//         .survivor_selector(ParamMapper::<C>::map(mapper, &py_builder.survivor_selector)),
//     SelectorConfig::Rank(mapper) => engine_builder
//         .survivor_selector(ParamMapper::<C>::map(mapper, &py_builder.survivor_selector)),
//     SelectorConfig::Elitism(mapper) => engine_builder
//         .survivor_selector(ParamMapper::<C>::map(mapper, &py_builder.survivor_selector)),
//     SelectorConfig::Boltzmann(mapper) => engine_builder
//         .survivor_selector(ParamMapper::<C>::map(mapper, &py_builder.survivor_selector)),
//     SelectorConfig::StochasticUniversalSampling(mapper) => engine_builder
//         .survivor_selector(ParamMapper::<C>::map(mapper, &py_builder.survivor_selector)),
//     SelectorConfig::LinearRank(mapper) => engine_builder
//         .survivor_selector(ParamMapper::<C>::map(mapper, &py_builder.survivor_selector)),
//     SelectorConfig::NSGA2(mapper) => engine_builder
//         .survivor_selector(ParamMapper::<C>::map(mapper, &py_builder.survivor_selector)),
// };

// let l = match offspring_selector_type {
//     SelectorConfig::Tournament(mapper) => t.offspring_selector(ParamMapper::<C>::map(
//         mapper,
//         &py_builder.offspring_selector,
//     )),
//     SelectorConfig::Roulette(mapper) => t.offspring_selector(ParamMapper::<C>::map(
//         mapper,
//         &py_builder.offspring_selector,
//     )),
//     SelectorConfig::Rank(mapper) => t.offspring_selector(ParamMapper::<C>::map(
//         mapper,
//         &py_builder.offspring_selector,
//     )),
//     SelectorConfig::Elitism(mapper) => t.offspring_selector(ParamMapper::<C>::map(
//         mapper,
//         &py_builder.offspring_selector,
//     )),
//     SelectorConfig::Boltzmann(mapper) => t.offspring_selector(ParamMapper::<C>::map(
//         mapper,
//         &py_builder.offspring_selector,
//     )),
//     SelectorConfig::StochasticUniversalSampling(mapper) => t.offspring_selector(
//         ParamMapper::<C>::map(mapper, &py_builder.offspring_selector),
//     ),
//     SelectorConfig::LinearRank(mapper) => t.offspring_selector(ParamMapper::<C>::map(
//         mapper,
//         &py_builder.offspring_selector,
//     )),
//     SelectorConfig::NSGA2(mapper) => t.offspring_selector(ParamMapper::<C>::map(
//         mapper,
//         &py_builder.offspring_selector,
//     )),
// };
