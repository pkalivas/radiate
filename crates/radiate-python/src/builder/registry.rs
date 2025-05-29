use super::{
    AlterConfig, AlterType, ParamMapper, alter_name_to_type,
    selector::{
        BoltzmannSelectorMapper, EliteSelectorMapper, LinearRankSelectorMapper,
        NSGA2SelectorMapper, RankSelectorMapper, RouletteSelectorMapper, SelectorConfig,
        SelectorType, StochasticUniversalSamplingSelectorMapper, TournamentSelectorMapper,
        selector_name_to_type,
    },
};
use crate::{PyEngineParam, PyGeneType};
use radiate::{
    Alter, BitChromosome, CharChromosome, Chromosome, FloatChromosome, IntChromosome, Select,
    selector,
};
use std::collections::HashMap;

pub struct EngineRegistry {
    pub alters: HashMap<PyGeneType, HashMap<AlterType, AlterConfig>>,
    pub selectors: HashMap<SelectorType, SelectorConfig>,
}

impl EngineRegistry {
    pub fn new() -> Self {
        let mut registry = EngineRegistry {
            alters: HashMap::new(),
            selectors: HashMap::new(),
        };

        Self::register_float_alters(&mut registry);
        Self::register_int_alters(&mut registry);
        Self::register_char_alters(&mut registry);
        Self::register_bit_alters(&mut registry);

        registry
    }

    pub fn get_selectors<C: Chromosome + Clone>(&self, params: PyEngineParam) {
        //-> impl Select<C> {
        let selector_config = self.selectors.get(&selector_name_to_type(params.name()));
        if let Some(config) = selector_config {
            // match config {
            //     SelectorConfig::Tournament(mapper) => mapper.map(&params),
            //     SelectorConfig::Roulette(mapper) => mapper.map(&params),
            //     SelectorConfig::Rank(mapper) => mapper.map(&params),
            //     SelectorConfig::Elitism(mapper) => mapper.map(&params),
            //     SelectorConfig::Boltzmann(mapper) => mapper.map(&params),
            //     SelectorConfig::StochasticUniversalSampling(mapper) => mapper.map(&params),
            //     SelectorConfig::LinearRank(mapper) => mapper.map(&params),
            //     SelectorConfig::NSGA2(mapper) => mapper.map(&params),
            // }
        } else {
            panic!("Unknown selector type")
        }
    }

    pub fn get_alters<C: Chromosome + Clone + 'static>(
        &self,
        gene_type: PyGeneType,
        params: &[PyEngineParam],
    ) -> Vec<Box<dyn Alter<C>>> {
        let mut result = Vec::new();
        let alters_map = self.alters.get(&gene_type);

        if let Some(alters) = alters_map {
            for param in params {
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

        result
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

    fn transmute_alter<C: Chromosome + 'static, T: Chromosome>(
        alter: Box<dyn Alter<C>>,
    ) -> Box<dyn Alter<T>> {
        unsafe { std::mem::transmute::<Box<dyn Alter<C>>, Box<dyn Alter<T>>>(alter) }
    }
}
