use crate::{
    ParamMapper, PyEngineBuilder, PyEngineParam, PyGeneType, registry::registry::ComponentRegistry,
};
use core::panic;
use radiate::prelude::*;
use std::{collections::HashMap, hash::Hash};

use super::register::{
    register_bit_alters, register_char_alters, register_float_alters, register_int_alters,
};

const BLEND_CROSSOVER: &str = "blend_crossover";
const INTERMEDIATE_CROSSOVER: &str = "intermediate_crossover";
const UNIFORM_CROSSOVER: &str = "uniform_crossover";
const MEAN_CROSSOVER: &str = "mean_crossover";
const SHUFFLE_CROSSOVER: &str = "shuffle_crossover";
const MULTI_POINT_CROSSOVER: &str = "multi_point_crossover";
const SIMULATED_BINARY_CROSSOVER: &str = "simulated_binary_crossover";
const UNIFORM_MUTATOR: &str = "uniform_mutator";
const ARITHMETIC_MUTATOR: &str = "arithmetic_mutator";
const GAUSSIAN_MUTATOR: &str = "gaussian_mutator";
const SCRAMBLE_MUTATOR: &str = "scramble_mutator";
const SWAP_MUTATOR: &str = "swap_mutator";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlterType {
    // Crossovers
    BlendCrossover,
    IntermediateCrossover,
    UniformCrossover,
    MeanCrossover,
    ShuffleCrossover,
    MultiPointCrossover,
    SimulatedBinaryCrossover,

    // Mutators
    UniformMutator,
    ArithmeticMutator,
    GaussianMutator,
    ScrambleMutator,
    SwapMutator,
}

pub(super) fn alter_name_to_type(name: &str) -> AlterType {
    match name {
        BLEND_CROSSOVER => AlterType::BlendCrossover,
        INTERMEDIATE_CROSSOVER => AlterType::IntermediateCrossover,
        UNIFORM_CROSSOVER => AlterType::UniformCrossover,
        MEAN_CROSSOVER => AlterType::MeanCrossover,
        SHUFFLE_CROSSOVER => AlterType::ShuffleCrossover,
        MULTI_POINT_CROSSOVER => AlterType::MultiPointCrossover,
        SIMULATED_BINARY_CROSSOVER => AlterType::SimulatedBinaryCrossover,
        UNIFORM_MUTATOR => AlterType::UniformMutator,
        ARITHMETIC_MUTATOR => AlterType::ArithmeticMutator,
        GAUSSIAN_MUTATOR => AlterType::GaussianMutator,
        SCRAMBLE_MUTATOR => AlterType::ScrambleMutator,
        SWAP_MUTATOR => AlterType::SwapMutator,
        _ => panic!("Unknown alter type: {}", name),
    }
}

enum AlterTransform {
    Float(Box<dyn Fn(&PyEngineParam) -> Vec<Box<dyn Alter<FloatChromosome> + 'static>>>),
    Int(Box<dyn Fn(&PyEngineParam) -> Vec<Box<dyn Alter<IntChromosome<i32>> + 'static>>>),
    Char(Box<dyn Fn(&PyEngineParam) -> Vec<Box<dyn Alter<CharChromosome> + 'static>>>),
    Bit(Box<dyn Fn(&PyEngineParam) -> Vec<Box<dyn Alter<BitChromosome> + 'static>>>),
}

pub struct AlterRegistry {
    alters: HashMap<PyGeneType, HashMap<AlterType, AlterTransform>>,
}

impl AlterRegistry {
    pub fn new() -> Self {
        let mut registry = AlterRegistry {
            alters: HashMap::new(),
        };

        register_float_alters(&mut registry);
        register_int_alters(&mut registry);
        register_char_alters(&mut registry);
        register_bit_alters(&mut registry);
        registry
    }

    pub fn register_alter_mapper<
        C: Chromosome + 'static,
        M: ParamMapper<Output = Vec<Box<dyn Alter<C>>>> + 'static,
    >(
        &mut self,
        gene_type: PyGeneType,
        alter_type: AlterType,
        mapper: M,
    ) {
        let alter_config = match gene_type {
            PyGeneType::Float => AlterTransform::Float(Box::new(move |param| {
                mapper
                    .map(param)
                    .into_iter()
                    .map(|alter| transmute_alter::<C, FloatChromosome>(alter))
                    .collect::<Vec<_>>()
            })),
            PyGeneType::Int => AlterTransform::Int(Box::new(move |param| {
                mapper
                    .map(param)
                    .into_iter()
                    .map(|alter| transmute_alter::<C, IntChromosome<i32>>(alter))
                    .collect::<Vec<_>>()
            })),
            PyGeneType::Char => AlterTransform::Char(Box::new(move |param| {
                mapper
                    .map(param)
                    .into_iter()
                    .map(|alter| transmute_alter::<C, CharChromosome>(alter))
                    .collect::<Vec<_>>()
            })),
            PyGeneType::Bit => AlterTransform::Bit(Box::new(move |param| {
                mapper
                    .map(param)
                    .into_iter()
                    .map(|alter| transmute_alter::<C, BitChromosome>(alter))
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
}

impl ComponentRegistry for AlterRegistry {
    fn apply<C, T, E>(
        &self,
        engine_builder: GeneticEngineBuilder<C, T, E>,
        py_builder: &PyEngineBuilder,
        gene_type: PyGeneType,
    ) -> GeneticEngineBuilder<C, T, E>
    where
        C: Chromosome + Clone + PartialEq + 'static,
        T: Clone + Send + Sync + 'static,
        E: Epoch<Chromosome = C> + 'static,
    {
        let mut result = Vec::new();
        let alters_map = self.alters.get(&gene_type);

        if let Some(alters) = alters_map {
            for param in py_builder.alters.iter() {
                let alter_type = alter_name_to_type(param.name());
                if let Some(alter_config) = alters.get(&alter_type) {
                    result.extend(match alter_config {
                        AlterTransform::Float(mapper) => mapper(param)
                            .into_iter()
                            .map(|alter| transmute_alter::<FloatChromosome, C>(alter))
                            .collect::<Vec<_>>(),
                        AlterTransform::Int(mapper) => mapper(param)
                            .into_iter()
                            .map(|alter| transmute_alter::<IntChromosome<i32>, C>(alter))
                            .collect::<Vec<_>>(),
                        AlterTransform::Char(mapper) => mapper(param)
                            .into_iter()
                            .map(|alter| transmute_alter::<CharChromosome, C>(alter))
                            .collect::<Vec<_>>(),
                        AlterTransform::Bit(mapper) => mapper(param)
                            .into_iter()
                            .map(|alter| transmute_alter::<BitChromosome, C>(alter))
                            .collect::<Vec<_>>(),
                    });
                }
            }
        }

        engine_builder.alter(result)
    }
}

fn transmute_alter<C: Chromosome + 'static, T: Chromosome>(
    alter: Box<dyn Alter<C>>,
) -> Box<dyn Alter<T>> {
    // SAFETY: This transmute is safe because we ensure that the Alter<C> can be used as Alter<T> during creation,
    // its not even possible to register the alter if C and T are not compatible.
    unsafe { std::mem::transmute::<Box<dyn Alter<C>>, Box<dyn Alter<T>>>(alter) }
}
