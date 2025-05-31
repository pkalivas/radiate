use super::{EngineRegistry, ParamMapper};
use crate::{PyEngineParam, PyGeneType};
use core::panic;
use radiate::prelude::*;
use std::hash::Hash;

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

pub enum AlterConfig {
    Float(Box<dyn Fn(&PyEngineParam) -> Vec<Box<dyn Alter<FloatChromosome> + 'static>>>),
    Int(Box<dyn Fn(&PyEngineParam) -> Vec<Box<dyn Alter<IntChromosome<i32>> + 'static>>>),
    Char(Box<dyn Fn(&PyEngineParam) -> Vec<Box<dyn Alter<CharChromosome> + 'static>>>),
    Bit(Box<dyn Fn(&PyEngineParam) -> Vec<Box<dyn Alter<BitChromosome> + 'static>>>),
}

impl EngineRegistry {
    pub fn register_float_alters(registry: &mut EngineRegistry) {
        registry.register_alter_mapper::<FloatChromosome, BlendCrossoverMapper>(
            PyGeneType::Float,
            AlterType::BlendCrossover,
            BlendCrossoverMapper,
        );
        registry.register_alter_mapper::<FloatChromosome, IntermediateCrossoverMapper>(
            PyGeneType::Float,
            AlterType::IntermediateCrossover,
            IntermediateCrossoverMapper,
        );
        registry.register_alter_mapper::<FloatChromosome, UniformCrossoverMapper>(
            PyGeneType::Float,
            AlterType::UniformCrossover,
            UniformCrossoverMapper,
        );
        registry.register_alter_mapper::<FloatChromosome, MeanCrossoverMapper>(
            PyGeneType::Float,
            AlterType::MeanCrossover,
            MeanCrossoverMapper,
        );
        registry.register_alter_mapper::<FloatChromosome, ShuffleCrossoverMapper>(
            PyGeneType::Float,
            AlterType::ShuffleCrossover,
            ShuffleCrossoverMapper,
        );
        registry.register_alter_mapper::<FloatChromosome, SimulatedBinaryCrossoverMapper>(
            PyGeneType::Float,
            AlterType::SimulatedBinaryCrossover,
            SimulatedBinaryCrossoverMapper,
        );
        registry.register_alter_mapper::<FloatChromosome, MultiPointCrossoverMapper>(
            PyGeneType::Float,
            AlterType::MultiPointCrossover,
            MultiPointCrossoverMapper,
        );
        registry.register_alter_mapper::<FloatChromosome, UniformMutatorMapper>(
            PyGeneType::Float,
            AlterType::UniformMutator,
            UniformMutatorMapper,
        );
        registry.register_alter_mapper::<FloatChromosome, ArithmeticMutatorMapper>(
            PyGeneType::Float,
            AlterType::ArithmeticMutator,
            ArithmeticMutatorMapper,
        );
        registry.register_alter_mapper::<FloatChromosome, GaussianMutatorMapper>(
            PyGeneType::Float,
            AlterType::GaussianMutator,
            GaussianMutatorMapper,
        );
        registry.register_alter_mapper::<FloatChromosome, ScrambleMutatorMapper>(
            PyGeneType::Float,
            AlterType::ScrambleMutator,
            ScrambleMutatorMapper,
        );
        registry.register_alter_mapper::<FloatChromosome, SwapMutatorMapper>(
            PyGeneType::Float,
            AlterType::SwapMutator,
            SwapMutatorMapper,
        );
    }

    pub fn register_int_alters(registry: &mut EngineRegistry) {
        registry.register_alter_mapper::<IntChromosome<i32>, MultiPointCrossoverMapper>(
            PyGeneType::Int,
            AlterType::MultiPointCrossover,
            MultiPointCrossoverMapper,
        );
        registry.register_alter_mapper::<IntChromosome<i32>, UniformCrossoverMapper>(
            PyGeneType::Int,
            AlterType::UniformCrossover,
            UniformCrossoverMapper,
        );
        registry.register_alter_mapper::<IntChromosome<i32>, UniformMutatorMapper>(
            PyGeneType::Int,
            AlterType::UniformMutator,
            UniformMutatorMapper,
        );
        registry.register_alter_mapper::<IntChromosome<i32>, ArithmeticMutatorMapper>(
            PyGeneType::Int,
            AlterType::ArithmeticMutator,
            ArithmeticMutatorMapper,
        );
        registry.register_alter_mapper::<IntChromosome<i32>, MeanCrossoverMapper>(
            PyGeneType::Int,
            AlterType::MeanCrossover,
            MeanCrossoverMapper,
        );
        registry.register_alter_mapper::<IntChromosome<i32>, ShuffleCrossoverMapper>(
            PyGeneType::Int,
            AlterType::ShuffleCrossover,
            ShuffleCrossoverMapper,
        );
        registry.register_alter_mapper::<IntChromosome<i32>, ScrambleMutatorMapper>(
            PyGeneType::Int,
            AlterType::ScrambleMutator,
            ScrambleMutatorMapper,
        );
        registry.register_alter_mapper::<IntChromosome<i32>, SwapMutatorMapper>(
            PyGeneType::Int,
            AlterType::SwapMutator,
            SwapMutatorMapper,
        );
    }

    pub fn register_char_alters(registry: &mut EngineRegistry) {
        registry.register_alter_mapper::<CharChromosome, MultiPointCrossoverMapper>(
            PyGeneType::Char,
            AlterType::MultiPointCrossover,
            MultiPointCrossoverMapper,
        );
        registry.register_alter_mapper::<CharChromosome, UniformCrossoverMapper>(
            PyGeneType::Char,
            AlterType::UniformCrossover,
            UniformCrossoverMapper,
        );
        registry.register_alter_mapper::<CharChromosome, UniformMutatorMapper>(
            PyGeneType::Char,
            AlterType::UniformMutator,
            UniformMutatorMapper,
        );
        registry.register_alter_mapper::<CharChromosome, ShuffleCrossoverMapper>(
            PyGeneType::Char,
            AlterType::ShuffleCrossover,
            ShuffleCrossoverMapper,
        );
        registry.register_alter_mapper::<CharChromosome, ScrambleMutatorMapper>(
            PyGeneType::Char,
            AlterType::ScrambleMutator,
            ScrambleMutatorMapper,
        );
        registry.register_alter_mapper::<CharChromosome, SwapMutatorMapper>(
            PyGeneType::Char,
            AlterType::SwapMutator,
            SwapMutatorMapper,
        );
    }

    pub fn register_bit_alters(registry: &mut EngineRegistry) {
        registry.register_alter_mapper::<BitChromosome, MultiPointCrossoverMapper>(
            PyGeneType::Bit,
            AlterType::MultiPointCrossover,
            MultiPointCrossoverMapper,
        );
        registry.register_alter_mapper::<BitChromosome, UniformCrossoverMapper>(
            PyGeneType::Bit,
            AlterType::UniformCrossover,
            UniformCrossoverMapper,
        );
        registry.register_alter_mapper::<BitChromosome, UniformMutatorMapper>(
            PyGeneType::Bit,
            AlterType::UniformMutator,
            UniformMutatorMapper,
        );
        registry.register_alter_mapper::<BitChromosome, ShuffleCrossoverMapper>(
            PyGeneType::Bit,
            AlterType::ShuffleCrossover,
            ShuffleCrossoverMapper,
        );
        registry.register_alter_mapper::<BitChromosome, ScrambleMutatorMapper>(
            PyGeneType::Bit,
            AlterType::ScrambleMutator,
            ScrambleMutatorMapper,
        );
        registry.register_alter_mapper::<BitChromosome, SwapMutatorMapper>(
            PyGeneType::Bit,
            AlterType::SwapMutator,
            SwapMutatorMapper,
        );
    }
}

struct BlendCrossoverMapper;

impl<C> ParamMapper<C> for BlendCrossoverMapper
where
    C: Chromosome<Gene = FloatGene> + 'static,
{
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let alpha = param
            .get_arg("alpha")
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        let rate = param
            .get_arg("rate")
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(BlendCrossover::new(rate, alpha))
    }
}

struct IntermediateCrossoverMapper;

impl ParamMapper<FloatChromosome> for IntermediateCrossoverMapper {
    type Output = Vec<Box<dyn Alter<FloatChromosome>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate")
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        let alpha = param
            .get_arg("alpha")
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(IntermediateCrossover::new(rate, alpha))
    }
}

struct UniformCrossoverMapper;

impl<C: Chromosome + 'static> ParamMapper<C> for UniformCrossoverMapper {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(UniformCrossover::new(rate))
    }
}

struct MeanCrossoverMapper;

impl<C: Chromosome + 'static> ParamMapper<C> for MeanCrossoverMapper
where
    C::Gene: ArithmeticGene,
{
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(MeanCrossover::new(rate))
    }
}

struct ShuffleCrossoverMapper;

impl<C: Chromosome + Clone + 'static> ParamMapper<C> for ShuffleCrossoverMapper {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(ShuffleCrossover::new(rate))
    }
}

struct SimulatedBinaryCrossoverMapper;

impl<C: Chromosome<Gene = FloatGene> + 'static> ParamMapper<C> for SimulatedBinaryCrossoverMapper {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        let contiguty = param
            .get_arg("contiguty".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(SimulatedBinaryCrossover::new(contiguty, rate))
    }
}

struct MultiPointCrossoverMapper;

impl<C: Chromosome + 'static> ParamMapper<C> for MultiPointCrossoverMapper {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        let points = param
            .get_arg("num_points".into())
            .map(|s| s.parse::<usize>().unwrap())
            .unwrap_or(2);
        alters!(MultiPointCrossover::new(rate, points))
    }
}

struct UniformMutatorMapper;

impl<C: Chromosome + 'static> ParamMapper<C> for UniformMutatorMapper {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(UniformMutator::new(rate))
    }
}

struct ArithmeticMutatorMapper;

impl<C: Chromosome + 'static> ParamMapper<C> for ArithmeticMutatorMapper
where
    C::Gene: ArithmeticGene,
{
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(ArithmeticMutator::new(rate))
    }
}

struct GaussianMutatorMapper;

impl<C: Chromosome<Gene = FloatGene> + 'static> ParamMapper<C> for GaussianMutatorMapper {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(GaussianMutator::new(rate))
    }
}

struct ScrambleMutatorMapper;

impl<C: Chromosome + 'static> ParamMapper<C> for ScrambleMutatorMapper {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(ScrambleMutator::new(rate))
    }
}

struct SwapMutatorMapper;

impl<C: Chromosome + 'static> ParamMapper<C> for SwapMutatorMapper {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(SwapMutator::new(rate))
    }
}
