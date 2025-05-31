use super::{EngineRegistry, ParamMapper};
use crate::{PyEngineParam, PyGeneType};
use radiate::{
    ArithmeticGene, BitChromosome, BitGene, CharChromosome, CharGene, Chromosome, Diversity,
    EuclideanDistance, FloatChromosome, FloatGene, Gene, HammingDistance, IntChromosome, IntGene,
};
use std::sync::Arc;

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
pub enum DiversityConfig {
    Int(Box<dyn Fn(&PyEngineParam) -> Arc<dyn Diversity<IntChromosome<i32>>>>),
    Float(Box<dyn Fn(&PyEngineParam) -> Arc<dyn Diversity<FloatChromosome>>>),
    Char(Box<dyn Fn(&PyEngineParam) -> Arc<dyn Diversity<CharChromosome>>>),
    Bit(Box<dyn Fn(&PyEngineParam) -> Arc<dyn Diversity<BitChromosome>>>),
}

impl EngineRegistry {
    pub fn register_int_diversity_mappers(registry: &mut EngineRegistry) {
        registry.register_diversity_mapper::<IntChromosome<i32>, HammingDistanceDiversityMapper<IntGene<i32>>>(
            PyGeneType::Int,
            DiversityType::HammingDistance,
            HammingDistanceDiversityMapper::<IntGene<i32>>::default(),
        );
    }

    pub fn register_float_diversity_mappers(registry: &mut EngineRegistry) {
        registry.register_diversity_mapper::<FloatChromosome, HammingDistanceDiversityMapper<FloatGene>>(
            PyGeneType::Float,
            DiversityType::HammingDistance,
            HammingDistanceDiversityMapper::<FloatGene>::default(),
        );

        registry.register_diversity_mapper::<FloatChromosome, EuclideanDistanceDiversityMapper<FloatGene>>(
            PyGeneType::Float,
            DiversityType::EuclideanDistance,
            EuclideanDistanceDiversityMapper::<FloatGene>::default(),
        );
    }

    pub fn register_char_diversity_mappers(registry: &mut EngineRegistry) {
        registry
            .register_diversity_mapper::<CharChromosome, HammingDistanceDiversityMapper<CharGene>>(
                PyGeneType::Char,
                DiversityType::HammingDistance,
                HammingDistanceDiversityMapper::<CharGene>::default(),
            );
    }

    pub fn register_bit_diversity_mappers(registry: &mut EngineRegistry) {
        registry
            .register_diversity_mapper::<BitChromosome, HammingDistanceDiversityMapper<BitGene>>(
                PyGeneType::Bit,
                DiversityType::HammingDistance,
                HammingDistanceDiversityMapper::<BitGene>::default(),
            );
    }
}

pub struct HammingDistanceDiversityMapper<G: Gene>
where
    G::Allele: PartialEq,
{
    _marker: std::marker::PhantomData<G>,
}

impl<G, C> ParamMapper<C> for HammingDistanceDiversityMapper<G>
where
    C: Chromosome<Gene = G>,
    G: Gene,
    G::Allele: PartialEq,
{
    type Output = Arc<dyn Diversity<C>>;
    fn map(&self, _: &PyEngineParam) -> Self::Output {
        Arc::new(HammingDistance)
    }
}

impl<G: Gene + Default> Default for HammingDistanceDiversityMapper<G>
where
    G::Allele: PartialEq,
{
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

pub struct EuclideanDistanceDiversityMapper<G: Gene>
where
    G::Allele: Into<f32> + Copy,
{
    _marker: std::marker::PhantomData<G>,
}

impl<G, C> ParamMapper<C> for EuclideanDistanceDiversityMapper<G>
where
    C: Chromosome<Gene = G>,
    G: Gene + ArithmeticGene,
    G::Allele: Into<f32> + Copy,
{
    type Output = Arc<dyn Diversity<C>>;
    fn map(&self, _: &PyEngineParam) -> Self::Output {
        Arc::new(EuclideanDistance)
    }
}

impl<G: Gene + Default> Default for EuclideanDistanceDiversityMapper<G>
where
    G::Allele: Into<f32> + Copy,
{
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}
