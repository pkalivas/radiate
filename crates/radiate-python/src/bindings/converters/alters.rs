use crate::{
    PyEngineInput, PyEngineInputType, PyExpr,
    bindings::converters::{FromPyEngineInput, Registry},
    registry,
};
use radiate::{chromosomes::NumericAllele, *};
use radiate_utils::{Float, Integer};

pub(crate) fn int_alterers<I>() -> Registry<Alterer<IntChromosome<I>>>
where
    I: Integer,
{
    use crate::constants::components;
    registry!("alterer", |a| a.into_alterer(); {
        components::MULTI_POINT_CROSSOVER   => MultiPointCrossover,
        components::UNIFORM_CROSSOVER       => UniformCrossover,
        components::MEAN_CROSSOVER          => MeanCrossover,
        components::SHUFFLE_CROSSOVER       => ShuffleCrossover,

        components::ARITHMETIC_MUTATOR      => ArithmeticMutator,
        components::SWAP_MUTATOR            => SwapMutator,
        components::SCRAMBLE_MUTATOR        => ScrambleMutator,
        components::UNIFORM_MUTATOR         => UniformMutator,
        components::INVERSION_MUTATOR       => InversionMutator,
    })
}

pub(crate) fn float_alterers<F>() -> Registry<Alterer<FloatChromosome<F>>>
where
    F: Float + NumericAllele,
{
    use crate::constants::components;
    registry!("alterer", |a| a.into_alterer(); {
        components::MULTI_POINT_CROSSOVER        => MultiPointCrossover,
        components::UNIFORM_CROSSOVER            => UniformCrossover,
        components::MEAN_CROSSOVER               => MeanCrossover,
        components::INTERMEDIATE_CROSSOVER       => IntermediateCrossover,
        components::BLEND_CROSSOVER              => BlendCrossover,
        components::SIMULATED_BINARY_CROSSOVER   => SimulatedBinaryCrossover,
        components::DIFFERENTIAL_CROSSOVER       => DifferentialCrossover,

        components::GAUSSIAN_MUTATOR             => GaussianMutator,
        components::ARITHMETIC_MUTATOR           => ArithmeticMutator,
        components::SWAP_MUTATOR                 => SwapMutator,
        components::SCRAMBLE_MUTATOR             => ScrambleMutator,
        components::UNIFORM_MUTATOR              => UniformMutator,
        components::INVERSION_MUTATOR            => InversionMutator,
        components::POLYNOMIAL_MUTATOR           => PolynomialMutator,
        components::JITTER_MUTATOR               => JitterMutator,
    })
}

pub(crate) fn char_alterers() -> Registry<Alterer<CharChromosome>> {
    use crate::constants::components;
    registry!("alterer", |a| a.into_alterer(); {
        components::MULTI_POINT_CROSSOVER   => MultiPointCrossover,
        components::UNIFORM_CROSSOVER       => UniformCrossover,
        components::SHUFFLE_CROSSOVER       => ShuffleCrossover,

        components::SWAP_MUTATOR            => SwapMutator,
        components::SCRAMBLE_MUTATOR        => ScrambleMutator,
        components::UNIFORM_MUTATOR         => UniformMutator,
        components::INVERSION_MUTATOR       => InversionMutator,
    })
}

pub(crate) fn bit_alterers() -> Registry<Alterer<BitChromosome>> {
    use crate::constants::components;
    registry!("alterer", |a| a.into_alterer(); {
        components::MULTI_POINT_CROSSOVER   => MultiPointCrossover,
        components::UNIFORM_CROSSOVER       => UniformCrossover,
        components::SHUFFLE_CROSSOVER       => ShuffleCrossover,

        components::SWAP_MUTATOR            => SwapMutator,
        components::SCRAMBLE_MUTATOR        => ScrambleMutator,
        components::UNIFORM_MUTATOR         => UniformMutator,
        components::INVERSION_MUTATOR       => InversionMutator,
        components::BIT_FLIP_MUTATOR        => BitFlipMutator,
    })
}

pub(crate) fn packed_bit_alterers() -> Registry<Alterer<PackedBitChromosome>> {
    use crate::constants::components;
    registry!("alterer", |a| a.into_alterer(); {
        components::MULTI_POINT_CROSSOVER   => PackedBitMultiPointCrossover,
        components::UNIFORM_CROSSOVER       => PackedBitCrossover,

        components::BIT_FLIP_MUTATOR        => BitFlipMutator,
    })
}

pub(crate) fn perm_alterers() -> Registry<Alterer<PermutationChromosome<usize>>> {
    use crate::constants::components;
    registry!("alterer", |a| a.into_alterer(); {
        components::PARTIALLY_MAPPED_CROSSOVER => PMXCrossover,
        components::EDGE_RECOMBINE_CROSSOVER   => EdgeRecombinationCrossover,
        components::ORDER_CROSSOVER            => OrderCrossover,

        components::SWAP_MUTATOR               => SwapMutator,
        components::SCRAMBLE_MUTATOR           => ScrambleMutator,
        components::UNIFORM_MUTATOR            => UniformMutator,
        components::INVERSION_MUTATOR          => InversionMutator,
    })
}

pub(crate) fn graph_alterers<F: Float>() -> Registry<Alterer<GraphChromosome<Op<F>>>> {
    use crate::constants::components;
    registry!("alterer", |a| a.into_alterer(); {
        components::GRAPH_CROSSOVER       => GraphCrossover,

        components::GRAPH_MUTATOR         => GraphMutator,
        components::OPERATION_MUTATOR     => OperationMutator,
    })
}

pub(crate) fn tree_alterers<F: Float>() -> Registry<Alterer<TreeChromosome<Op<F>>>> {
    use crate::constants::components;
    registry!("alterer", |a| a.into_alterer(); {
        components::TREE_CROSSOVER        => TreeCrossover,

        components::HOIST_MUTATOR         => HoistMutator,
        components::OPERATION_MUTATOR     => OperationMutator,
        components::SUBTREE_MUTATOR       => SubtreeMutator,
    })
}

/// ---------------------------------------------------------------------------
/// Concrete converters
/// ---------------------------------------------------------------------------
impl FromPyEngineInput for SubtreeMutator {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        let max_depth = input
            .extract::<Option<i64>>("max_depth")?
            .map(|s| s as usize);

        if let Some(max_depth) = max_depth {
            Ok(SubtreeMutator::new(rate).with_max_depth(max_depth))
        } else {
            Ok(SubtreeMutator::new(rate))
        }
    }
}

impl FromPyEngineInput for DifferentialCrossover {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        let weight = input.extract::<f64>("weight")?;
        let crossover_rate = input.extract::<f64>("crossover_rate")?;
        Ok(DifferentialCrossover::new(
            rate,
            weight as f32,
            crossover_rate as f32,
        ))
    }
}

impl FromPyEngineInput for OrderCrossover {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        Ok(OrderCrossover::new(rate))
    }
}

impl FromPyEngineInput for JitterMutator {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        let magnitude = input.extract::<f64>("magnitude")?;
        Ok(JitterMutator::new(rate, magnitude as f32))
    }
}

impl FromPyEngineInput for InversionMutator {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        Ok(InversionMutator::new(rate))
    }
}

impl FromPyEngineInput for HoistMutator {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        Ok(HoistMutator::new(rate))
    }
}

impl FromPyEngineInput for TreeCrossover {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        let max_size = input.extract::<i64>("max_size")?;
        Ok(TreeCrossover::new(rate).with_max_size(max_size as usize))
    }
}

impl FromPyEngineInput for MultiPointCrossover {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        let points = input.extract::<i64>("num_points")?;
        Ok(MultiPointCrossover::new(rate, points as usize))
    }
}

impl FromPyEngineInput for PackedBitMultiPointCrossover {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        let points = input.extract::<i64>("num_points")?;
        Ok(PackedBitMultiPointCrossover::new(rate, points as usize))
    }
}

impl FromPyEngineInput for UniformCrossover {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        Ok(UniformCrossover::new(
            input.extract::<PyExpr>("rate")?.inner,
        ))
    }
}

impl FromPyEngineInput for PackedBitCrossover {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        Ok(PackedBitCrossover::new(rate))
    }
}

impl FromPyEngineInput for UniformMutator {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        Ok(UniformMutator::new(rate))
    }
}

impl FromPyEngineInput for MeanCrossover {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        Ok(MeanCrossover::new(rate))
    }
}

impl FromPyEngineInput for IntermediateCrossover {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        let alpha = input.extract::<f64>("alpha")?;
        Ok(IntermediateCrossover::new(rate, alpha as f32))
    }
}

impl FromPyEngineInput for BlendCrossover {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        let alpha = input.extract::<f64>("alpha")?;
        Ok(BlendCrossover::new(rate, alpha as f32))
    }
}

impl FromPyEngineInput for ShuffleCrossover {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        Ok(ShuffleCrossover::new(rate))
    }
}

impl FromPyEngineInput for PMXCrossover {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        Ok(PMXCrossover::new(rate))
    }
}

impl FromPyEngineInput for ScrambleMutator {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        Ok(ScrambleMutator::new(rate))
    }
}

impl FromPyEngineInput for SwapMutator {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        Ok(SwapMutator::new(rate))
    }
}

impl FromPyEngineInput for ArithmeticMutator {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        Ok(ArithmeticMutator::new(rate))
    }
}

impl FromPyEngineInput for GaussianMutator {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        Ok(GaussianMutator::new(rate))
    }
}

impl FromPyEngineInput for SimulatedBinaryCrossover {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        let contiguity = input.extract::<f64>("contiguity")?;
        Ok(SimulatedBinaryCrossover::new(rate, contiguity as f32))
    }
}

impl FromPyEngineInput for GraphCrossover {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;

        let rate = input.extract::<PyExpr>("rate")?.inner;
        let parent_node_rate = input.extract::<PyExpr>("parent_node_rate")?.inner;
        Ok(GraphCrossover::new(rate, parent_node_rate))
    }
}

impl FromPyEngineInput for GraphMutator {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let vertex_rate = input.extract::<PyExpr>("vertex_rate")?.inner;
        let edge_rate = input.extract::<PyExpr>("edge_rate")?.inner;
        let allow_recurrent = input.extract::<bool>("allow_recurrent")?;
        let target_size = input
            .extract::<Option<i64>>("target_size")?
            .map(|s| s as usize);

        let mutator = GraphMutator::new(vertex_rate, edge_rate).allow_recurrent(allow_recurrent);

        match target_size {
            Some(size) => Ok(mutator.target_size(size)),
            None => Ok(mutator),
        }
    }
}

impl FromPyEngineInput for OperationMutator {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        let replace_rate = input.extract::<PyExpr>("replace_rate")?.inner;
        Ok(OperationMutator::new(rate, replace_rate))
    }
}

impl FromPyEngineInput for EdgeRecombinationCrossover {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        Ok(EdgeRecombinationCrossover::new(rate))
    }
}

impl FromPyEngineInput for PolynomialMutator {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        let rate = input.extract::<PyExpr>("rate")?.inner;
        let eta = input.extract::<f64>("eta")?;
        Ok(PolynomialMutator::new(rate, eta as f32))
    }
}

impl FromPyEngineInput for BitFlipMutator {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Alterer)?;
        Ok(BitFlipMutator::new(input.extract::<PyExpr>("rate")?.inner))
    }
}
