use super::{mappers::*, register::register_selector_mappers};
use crate::{ParamMapper, PyEngineBuilder, PyGeneType, registry::registry::ComponentRegistry};
use radiate::{Chromosome, Epoch, GeneticEngineBuilder};
use std::collections::HashMap;

const TOURNAMENT_SELECTOR: &str = "tournament";
const ROULETTE_SELECTOR: &str = "roulette";
const RANK_SELECTOR: &str = "rank";
const ELITISM_SELECTOR: &str = "elitism";
const BOLTZMANN_SELECTOR: &str = "boltzmann";
const STOCHASTIC_UNIVERSAL_SAMPLING_SELECTOR: &str = "stochastic_universal_sampling";
const LINEAR_RANK_SELECTOR: &str = "linear_rank";
const NSGA2_SELECTOR: &str = "nsga2";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SelectorType {
    Tournament,
    Roulette,
    Rank,
    Elitism,
    Boltzmann,
    StochasticUniversalSampling,
    LinearRank,
    NSGA2,
}

pub enum SelectorConfig {
    Tournament(TournamentSelectorMapper),
    Roulette(RouletteSelectorMapper),
    Rank(RankSelectorMapper),
    Elitism(EliteSelectorMapper),
    Boltzmann(BoltzmannSelectorMapper),
    StochasticUniversalSampling(StochasticUniversalSamplingSelectorMapper),
    LinearRank(LinearRankSelectorMapper),
    NSGA2(NSGA2SelectorMapper),
}

pub fn selector_string_to_type(s: &str) -> Option<SelectorType> {
    match s {
        TOURNAMENT_SELECTOR => Some(SelectorType::Tournament),
        ROULETTE_SELECTOR => Some(SelectorType::Roulette),
        RANK_SELECTOR => Some(SelectorType::Rank),
        ELITISM_SELECTOR => Some(SelectorType::Elitism),
        BOLTZMANN_SELECTOR => Some(SelectorType::Boltzmann),
        STOCHASTIC_UNIVERSAL_SAMPLING_SELECTOR => Some(SelectorType::StochasticUniversalSampling),
        LINEAR_RANK_SELECTOR => Some(SelectorType::LinearRank),
        NSGA2_SELECTOR => Some(SelectorType::NSGA2),
        _ => None,
    }
}

pub struct SelectorRegistry {
    selectors: HashMap<SelectorType, SelectorConfig>,
}

impl SelectorRegistry {
    pub fn new() -> Self {
        let mut registry = SelectorRegistry {
            selectors: HashMap::new(),
        };

        register_selector_mappers(&mut registry);

        registry
    }

    pub fn register_selector(&mut self, selector_type: SelectorType, config: SelectorConfig) {
        self.selectors.insert(selector_type, config);
    }
}

impl ComponentRegistry for SelectorRegistry {
    fn apply<C, T, E>(
        &self,
        engine_builder: GeneticEngineBuilder<C, T, E>,
        py_builder: &PyEngineBuilder,
        _: PyGeneType,
    ) -> GeneticEngineBuilder<C, T, E>
    where
        C: Chromosome + Clone + PartialEq + 'static,
        T: Clone + Send + 'static,
        E: Epoch<Chromosome = C> + 'static,
    {
        let map_types = vec![
            (&py_builder.survivor_selector, false),
            (&py_builder.offspring_selector, true),
        ];

        let mut builder = engine_builder;

        for (selector_param, is_offspring) in map_types {
            match selector_string_to_type(selector_param.name()) {
                Some(selector_type) => {
                    if let Some(selector) = self.selectors.get(&selector_type) {
                        builder = match selector {
                            SelectorConfig::Tournament(mapper) => match is_offspring {
                                true => builder.offspring_selector(mapper.map(&selector_param)),
                                false => builder.survivor_selector(mapper.map(&selector_param)),
                            },
                            SelectorConfig::Roulette(mapper) => match is_offspring {
                                true => builder.offspring_selector(mapper.map(&selector_param)),
                                false => builder.survivor_selector(mapper.map(&selector_param)),
                            },
                            SelectorConfig::Rank(mapper) => match is_offspring {
                                true => builder.offspring_selector(mapper.map(&selector_param)),
                                false => builder.survivor_selector(mapper.map(&selector_param)),
                            },
                            SelectorConfig::Elitism(mapper) => match is_offspring {
                                true => builder.offspring_selector(mapper.map(&selector_param)),
                                false => builder.survivor_selector(mapper.map(&selector_param)),
                            },
                            SelectorConfig::Boltzmann(mapper) => match is_offspring {
                                true => builder.offspring_selector(mapper.map(&selector_param)),
                                false => builder.survivor_selector(mapper.map(&selector_param)),
                            },
                            SelectorConfig::StochasticUniversalSampling(mapper) => {
                                match is_offspring {
                                    true => builder.offspring_selector(mapper.map(&selector_param)),
                                    false => builder.survivor_selector(mapper.map(&selector_param)),
                                }
                            }
                            SelectorConfig::LinearRank(mapper) => match is_offspring {
                                true => builder.offspring_selector(mapper.map(&selector_param)),
                                false => builder.survivor_selector(mapper.map(&selector_param)),
                            },
                            SelectorConfig::NSGA2(mapper) => match is_offspring {
                                true => builder.offspring_selector(mapper.map(&selector_param)),
                                false => builder.survivor_selector(mapper.map(&selector_param)),
                            },
                        };
                    }
                }
                None => continue,
            };
        }

        builder
    }
}
