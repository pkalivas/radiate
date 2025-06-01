use super::{AlterRegistry, AlterType, mappers::*};
use crate::PyGeneType;
use radiate::prelude::*;

pub(super) fn register_float_alters(registry: &mut AlterRegistry) {
    registry.register_alter_mapper::<FloatChromosome, BlendCrossoverMapper<FloatChromosome>>(
        PyGeneType::Float,
        AlterType::BlendCrossover,
        BlendCrossoverMapper::default(),
    );
    registry
        .register_alter_mapper::<FloatChromosome, IntermediateCrossoverMapper<FloatChromosome>>(
            PyGeneType::Float,
            AlterType::IntermediateCrossover,
            IntermediateCrossoverMapper::default(),
        );
    registry.register_alter_mapper::<FloatChromosome, UniformCrossoverMapper<FloatChromosome>>(
        PyGeneType::Float,
        AlterType::UniformCrossover,
        UniformCrossoverMapper::default(),
    );
    registry
        .register_alter_mapper::<FloatChromosome, MeanCrossoverMapper<FloatGene, FloatChromosome>>(
            PyGeneType::Float,
            AlterType::MeanCrossover,
            MeanCrossoverMapper::default(),
        );
    registry.register_alter_mapper::<FloatChromosome, ShuffleCrossoverMapper<FloatChromosome>>(
        PyGeneType::Float,
        AlterType::ShuffleCrossover,
        ShuffleCrossoverMapper::default(),
    );
    registry
        .register_alter_mapper::<FloatChromosome, SimulatedBinaryCrossoverMapper<FloatChromosome>>(
            PyGeneType::Float,
            AlterType::SimulatedBinaryCrossover,
            SimulatedBinaryCrossoverMapper::default(),
        );
    registry.register_alter_mapper::<FloatChromosome, MultiPointCrossoverMapper<FloatChromosome>>(
        PyGeneType::Float,
        AlterType::MultiPointCrossover,
        MultiPointCrossoverMapper::default(),
    );
    registry.register_alter_mapper::<FloatChromosome, UniformMutatorMapper<FloatChromosome>>(
        PyGeneType::Float,
        AlterType::UniformMutator,
        UniformMutatorMapper::default(),
    );
    registry.register_alter_mapper::<FloatChromosome, ArithmeticMutatorMapper<FloatGene, FloatChromosome>>(
        PyGeneType::Float,
        AlterType::ArithmeticMutator,
        ArithmeticMutatorMapper::default(),
    );
    registry.register_alter_mapper::<FloatChromosome, GaussianMutatorMapper<FloatChromosome>>(
        PyGeneType::Float,
        AlterType::GaussianMutator,
        GaussianMutatorMapper::default(),
    );
    registry.register_alter_mapper::<FloatChromosome, ScrambleMutatorMapper<FloatChromosome>>(
        PyGeneType::Float,
        AlterType::ScrambleMutator,
        ScrambleMutatorMapper::default(),
    );
    registry.register_alter_mapper::<FloatChromosome, SwapMutatorMapper<FloatChromosome>>(
        PyGeneType::Float,
        AlterType::SwapMutator,
        SwapMutatorMapper::default(),
    );
}

pub(super) fn register_int_alters(registry: &mut AlterRegistry) {
    registry
        .register_alter_mapper::<IntChromosome<i32>, MultiPointCrossoverMapper<IntChromosome<i32>>>(
            PyGeneType::Int,
            AlterType::MultiPointCrossover,
            MultiPointCrossoverMapper::default(),
        );
    registry
        .register_alter_mapper::<IntChromosome<i32>, UniformCrossoverMapper<IntChromosome<i32>>>(
            PyGeneType::Int,
            AlterType::UniformCrossover,
            UniformCrossoverMapper::default(),
        );
    registry.register_alter_mapper::<IntChromosome<i32>, UniformMutatorMapper<IntChromosome<i32>>>(
        PyGeneType::Int,
        AlterType::UniformMutator,
        UniformMutatorMapper::default(),
    );
    registry
        .register_alter_mapper::<IntChromosome<i32>, ArithmeticMutatorMapper<IntGene<i32>, IntChromosome<i32>>>(
            PyGeneType::Int,
            AlterType::ArithmeticMutator,
            ArithmeticMutatorMapper::default(),
        );
    registry.register_alter_mapper::<IntChromosome<i32>, MeanCrossoverMapper<IntGene<i32>, IntChromosome<i32>>>(
        PyGeneType::Int,
        AlterType::MeanCrossover,
        MeanCrossoverMapper::default(),
    );
    registry
        .register_alter_mapper::<IntChromosome<i32>, ShuffleCrossoverMapper<IntChromosome<i32>>>(
            PyGeneType::Int,
            AlterType::ShuffleCrossover,
            ShuffleCrossoverMapper::default(),
        );
    registry
        .register_alter_mapper::<IntChromosome<i32>, ScrambleMutatorMapper<IntChromosome<i32>>>(
            PyGeneType::Int,
            AlterType::ScrambleMutator,
            ScrambleMutatorMapper::default(),
        );
    registry.register_alter_mapper::<IntChromosome<i32>, SwapMutatorMapper<IntChromosome<i32>>>(
        PyGeneType::Int,
        AlterType::SwapMutator,
        SwapMutatorMapper::default(),
    );
}

pub(super) fn register_char_alters(registry: &mut AlterRegistry) {
    registry.register_alter_mapper::<CharChromosome, MultiPointCrossoverMapper<CharChromosome>>(
        PyGeneType::Char,
        AlterType::MultiPointCrossover,
        MultiPointCrossoverMapper::default(),
    );
    registry.register_alter_mapper::<CharChromosome, UniformCrossoverMapper<CharChromosome>>(
        PyGeneType::Char,
        AlterType::UniformCrossover,
        UniformCrossoverMapper::default(),
    );
    registry.register_alter_mapper::<CharChromosome, UniformMutatorMapper<CharChromosome>>(
        PyGeneType::Char,
        AlterType::UniformMutator,
        UniformMutatorMapper::default(),
    );
    registry.register_alter_mapper::<CharChromosome, ShuffleCrossoverMapper<CharChromosome>>(
        PyGeneType::Char,
        AlterType::ShuffleCrossover,
        ShuffleCrossoverMapper::default(),
    );
    registry.register_alter_mapper::<CharChromosome, ScrambleMutatorMapper<CharChromosome>>(
        PyGeneType::Char,
        AlterType::ScrambleMutator,
        ScrambleMutatorMapper::default(),
    );
    registry.register_alter_mapper::<CharChromosome, SwapMutatorMapper<CharChromosome>>(
        PyGeneType::Char,
        AlterType::SwapMutator,
        SwapMutatorMapper::default(),
    );
}

pub(super) fn register_bit_alters(registry: &mut AlterRegistry) {
    registry.register_alter_mapper::<BitChromosome, MultiPointCrossoverMapper<BitChromosome>>(
        PyGeneType::Bit,
        AlterType::MultiPointCrossover,
        MultiPointCrossoverMapper::default(),
    );
    registry.register_alter_mapper::<BitChromosome, UniformCrossoverMapper<BitChromosome>>(
        PyGeneType::Bit,
        AlterType::UniformCrossover,
        UniformCrossoverMapper::default(),
    );
    registry.register_alter_mapper::<BitChromosome, UniformMutatorMapper<BitChromosome>>(
        PyGeneType::Bit,
        AlterType::UniformMutator,
        UniformMutatorMapper::default(),
    );
    registry.register_alter_mapper::<BitChromosome, ShuffleCrossoverMapper<BitChromosome>>(
        PyGeneType::Bit,
        AlterType::ShuffleCrossover,
        ShuffleCrossoverMapper::default(),
    );
    registry.register_alter_mapper::<BitChromosome, ScrambleMutatorMapper<BitChromosome>>(
        PyGeneType::Bit,
        AlterType::ScrambleMutator,
        ScrambleMutatorMapper::default(),
    );
    registry.register_alter_mapper::<BitChromosome, SwapMutatorMapper<BitChromosome>>(
        PyGeneType::Bit,
        AlterType::SwapMutator,
        SwapMutatorMapper::default(),
    );
}
