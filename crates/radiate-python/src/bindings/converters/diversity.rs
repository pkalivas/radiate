use crate::{
    PyEngineInput,
    bindings::converters::{FromPyEngineInput, Registry},
    registry,
};
use radiate::{
    BitChromosome, CharChromosome, CosineDistance, EuclideanDistance, FloatChromosome,
    GraphChromosome, HammingDistance, IntChromosome, NeatDistance, Op, PackedBitChromosome,
    PackedBitHammingDistance, PermutationChromosome, Phenotype, RadiateResult, TreeChromosome,
    TreeDistance, chromosomes::NumericAllele, distance::Distance, ops::OpFloat,
};
use radiate_utils::{Float, Integer};

type DistanceFor<C> = Box<dyn Distance<Phenotype<C>>>;

pub(crate) fn int_distances<I>() -> Registry<DistanceFor<IntChromosome<I>>>
where
    I: Integer + NumericAllele,
{
    use crate::constants::components;
    registry!("diversity measure", |d| Box::new(d) as DistanceFor<_>; {
        components::HAMMING_DISTANCE   => HammingDistance,
        components::EUCLIDEAN_DISTANCE => EuclideanDistance,
        components::COSINE_DISTANCE    => CosineDistance,
    })
}

pub(crate) fn float_distances<F>() -> Registry<DistanceFor<FloatChromosome<F>>>
where
    F: Float + NumericAllele,
{
    use crate::constants::components;
    registry!("diversity measure", |d| Box::new(d) as DistanceFor<_>; {
        components::HAMMING_DISTANCE   => HammingDistance,
        components::EUCLIDEAN_DISTANCE => EuclideanDistance,
        components::COSINE_DISTANCE    => CosineDistance,
    })
}

pub(crate) fn bit_distances() -> Registry<DistanceFor<BitChromosome>> {
    use crate::constants::components;
    registry!("diversity measure", |d| Box::new(d) as DistanceFor<_>; {
        components::HAMMING_DISTANCE => HammingDistance,
    })
}

pub(crate) fn packed_bit_distances() -> Registry<DistanceFor<PackedBitChromosome>> {
    use crate::constants::components;
    registry!("diversity measure", |d| Box::new(d) as DistanceFor<_>; {
        components::HAMMING_DISTANCE => PackedBitHammingDistance,
    })
}

pub(crate) fn char_distances() -> Registry<DistanceFor<CharChromosome>> {
    use crate::constants::components;
    registry!("diversity measure", |d| Box::new(d) as DistanceFor<_>; {
        components::HAMMING_DISTANCE => HammingDistance,
    })
}

pub(crate) fn perm_distances() -> Registry<DistanceFor<PermutationChromosome<usize>>> {
    use crate::constants::components;
    registry!("diversity measure", |d| Box::new(d) as DistanceFor<_>; {
        components::HAMMING_DISTANCE => HammingDistance,
    })
}

pub(crate) fn graph_distances<F: OpFloat>() -> Registry<DistanceFor<GraphChromosome<Op<F>>>> {
    use crate::constants::components;
    registry!("diversity measure", |d| Box::new(d) as DistanceFor<_>; {
        components::NEAT_DISTANCE    => NeatDistance,
        components::HAMMING_DISTANCE => HammingDistance,
    })
}

pub(crate) fn tree_distances<F: OpFloat>() -> Registry<DistanceFor<TreeChromosome<Op<F>>>> {
    use crate::constants::components;
    registry!("diversity measure", |d| Box::new(d) as DistanceFor<_>; {
        components::TREE_DISTANCE => TreeDistance,
    })
}

impl FromPyEngineInput for HammingDistance {
    fn from_input(_: &PyEngineInput) -> RadiateResult<Self> {
        Ok(HammingDistance)
    }
}

impl FromPyEngineInput for PackedBitHammingDistance {
    fn from_input(_: &PyEngineInput) -> RadiateResult<Self> {
        Ok(PackedBitHammingDistance)
    }
}

impl FromPyEngineInput for EuclideanDistance {
    fn from_input(_: &PyEngineInput) -> RadiateResult<Self> {
        Ok(EuclideanDistance)
    }
}

impl FromPyEngineInput for CosineDistance {
    fn from_input(_: &PyEngineInput) -> RadiateResult<Self> {
        Ok(CosineDistance)
    }
}

impl FromPyEngineInput for NeatDistance {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        let excess = input.extract::<f64>("excess")?;
        let disjoint = input.extract::<f64>("disjoint")?;
        let weight_diff = input.extract::<f64>("weight_diff")?;
        Ok(NeatDistance::new(
            excess as f32,
            disjoint as f32,
            weight_diff as f32,
        ))
    }
}

impl FromPyEngineInput for TreeDistance {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        match input.extract::<Option<f64>>("k")? {
            Some(k) => Ok(TreeDistance::new().with_k(k as f32)),
            None => Ok(TreeDistance::new()),
        }
    }
}
