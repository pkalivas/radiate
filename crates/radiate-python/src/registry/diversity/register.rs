use crate::PyGeneType;
use radiate::{
    BitChromosome, BitGene, CharChromosome, CharGene, FloatChromosome, FloatGene, IntChromosome,
    IntGene,
};

use super::{
    DiversityRegistry, DiversityType,
    mappers::{EuclideanDistanceDiversityMapper, HammingDistanceDiversityMapper},
};

pub(super) fn register_int_diversity_mappers(registry: &mut DiversityRegistry) {
    registry.register_diversity_mappers::<IntChromosome<i32>, HammingDistanceDiversityMapper<IntGene<i32>, IntChromosome<i32>>>(
            PyGeneType::Int,
            DiversityType::HammingDistance,
            HammingDistanceDiversityMapper::<IntGene<i32>, IntChromosome<i32>>::default(),
        );
}

pub(super) fn register_float_diversity_mappers(registry: &mut DiversityRegistry) {
    registry
        .register_diversity_mappers::<FloatChromosome, HammingDistanceDiversityMapper<FloatGene, FloatChromosome>>(
            PyGeneType::Float,
            DiversityType::HammingDistance,
            HammingDistanceDiversityMapper::<FloatGene, FloatChromosome>::default(),
        );

    registry
        .register_diversity_mappers::<FloatChromosome, EuclideanDistanceDiversityMapper<FloatGene, FloatChromosome>>(
            PyGeneType::Float,
            DiversityType::EuclideanDistance,
            EuclideanDistanceDiversityMapper::<FloatGene, FloatChromosome>::default(),
        );
}

pub(super) fn register_char_diversity_mappers(registry: &mut DiversityRegistry) {
    registry
        .register_diversity_mappers::<CharChromosome, HammingDistanceDiversityMapper<CharGene, CharChromosome>>(
            PyGeneType::Char,
            DiversityType::HammingDistance,
            HammingDistanceDiversityMapper::<CharGene, CharChromosome>::default(),
        );
}

pub(super) fn register_bit_diversity_mappers(registry: &mut DiversityRegistry) {
    registry.register_diversity_mappers::<BitChromosome, HammingDistanceDiversityMapper<BitGene, BitChromosome>>(
        PyGeneType::Bit,
        DiversityType::HammingDistance,
        HammingDistanceDiversityMapper::<BitGene, BitChromosome>::default(),
    );
}
