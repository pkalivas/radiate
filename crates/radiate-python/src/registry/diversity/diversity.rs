use crate::{
    ObjectValue, ParamMapper, PyEngineBuilder, PyEngineParam, PyGeneType,
    registry::registry::ComponentRegistry,
};
use radiate::{
    BitChromosome, CharChromosome, Chromosome, Diversity, Epoch, FloatChromosome,
    GeneticEngineBuilder, IntChromosome,
};
use std::{collections::HashMap, sync::Arc};

use super::register::{
    register_bit_diversity_mappers, register_char_diversity_mappers,
    register_float_diversity_mappers, register_int_diversity_mappers,
};

const HAMMING_DISTANCE: &str = "hamming_distance";
const EUCLIDEAN_DISTANCE: &str = "euclidean_distance";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DiversityType {
    HammingDistance,
    EuclideanDistance,
}

pub fn diversity_string_to_type(s: &str) -> Option<DiversityType> {
    match s {
        HAMMING_DISTANCE => Some(DiversityType::HammingDistance),
        EUCLIDEAN_DISTANCE => Some(DiversityType::EuclideanDistance),
        _ => None,
    }
}
enum DiversityTransform {
    Int(Box<dyn Fn(&PyEngineParam) -> Arc<dyn Diversity<IntChromosome<i32>>>>),
    Float(Box<dyn Fn(&PyEngineParam) -> Arc<dyn Diversity<FloatChromosome>>>),
    Char(Box<dyn Fn(&PyEngineParam) -> Arc<dyn Diversity<CharChromosome>>>),
    Bit(Box<dyn Fn(&PyEngineParam) -> Arc<dyn Diversity<BitChromosome>>>),
}

pub struct DiversityRegistry {
    diversity: HashMap<PyGeneType, HashMap<DiversityType, DiversityTransform>>,
}

impl DiversityRegistry {
    pub fn new() -> Self {
        let mut registry = DiversityRegistry {
            diversity: HashMap::new(),
        };

        register_int_diversity_mappers(&mut registry);
        register_float_diversity_mappers(&mut registry);
        register_char_diversity_mappers(&mut registry);
        register_bit_diversity_mappers(&mut registry);

        registry
    }

    pub fn register_diversity_mappers<C, M>(
        &mut self,
        gene_type: PyGeneType,
        diversity_type: DiversityType,
        mapper: M,
    ) where
        C: Chromosome + 'static,
        M: ParamMapper<Output = Arc<dyn Diversity<C>>> + 'static,
    {
        let config = match gene_type {
            PyGeneType::Float => DiversityTransform::Float(Box::new(move |param| {
                Self::transmute_diversity(mapper.map(param))
            })),
            PyGeneType::Int => DiversityTransform::Int(Box::new(move |param| {
                Self::transmute_diversity(mapper.map(param))
            })),
            PyGeneType::Char => DiversityTransform::Char(Box::new(move |param| {
                Self::transmute_diversity(mapper.map(param))
            })),
            PyGeneType::Bit => DiversityTransform::Bit(Box::new(move |param| {
                Self::transmute_diversity(mapper.map(param))
            })),
            _ => panic!("Unsupported gene type"),
        };

        self.diversity
            .entry(gene_type)
            .or_insert_with(HashMap::new)
            .insert(diversity_type, config);
    }

    fn transmute_diversity<C: Chromosome + 'static, T: Chromosome>(
        diversity: Arc<dyn Diversity<C>>,
    ) -> Arc<dyn Diversity<T>> {
        // SAFETY: This transmute is safe because we ensure that the Diversity<C> can be used as Diversity<T> during creation,
        // its not even possible to register the diversity if C and T are not compatible.
        unsafe { std::mem::transmute::<Arc<dyn Diversity<C>>, Arc<dyn Diversity<T>>>(diversity) }
    }
}

impl ComponentRegistry for DiversityRegistry {
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
        let diversity_type = py_builder
            .diversity
            .as_ref()
            .map(|diversity| diversity.name())
            .and_then(|name| diversity_string_to_type(name));

        if let Some(diversity_type) = diversity_type {
            if let Some(diversity_map) = self.diversity.get(&gene_type) {
                if let Some(diversity_config) = diversity_map.get(&diversity_type) {
                    let diversity_param = match diversity_config {
                        DiversityTransform::Float(mapper) => {
                            Self::transmute_diversity::<FloatChromosome, C>(mapper(
                                py_builder.diversity.as_ref().unwrap(),
                            ))
                        }
                        DiversityTransform::Int(mapper) => {
                            Self::transmute_diversity::<IntChromosome<i32>, C>(mapper(
                                py_builder.diversity.as_ref().unwrap(),
                            ))
                        }
                        DiversityTransform::Char(mapper) => {
                            Self::transmute_diversity::<CharChromosome, C>(mapper(
                                py_builder.diversity.as_ref().unwrap(),
                            ))
                        }
                        DiversityTransform::Bit(mapper) => {
                            Self::transmute_diversity::<BitChromosome, C>(mapper(
                                py_builder.diversity.as_ref().unwrap(),
                            ))
                        }
                    };

                    return engine_builder.with_values(|params| {
                        params.diversity = Some(diversity_param);
                        params.species_threshold = py_builder.species_threshold;
                        params.max_species_age = py_builder.max_species_age;
                    });
                }
            }
        }

        engine_builder
    }
}
