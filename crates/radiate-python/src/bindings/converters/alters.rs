use crate::{InputTransform, PyEngineInput, PyEngineInputType, PyExpr};
use radiate::{chromosomes::NumericAllele, ops::OpFloat, *};
use radiate_utils::{Float, Integer};

type AlterConv<C> = fn(&PyEngineInput) -> RadiateResult<Alterer<C>>;

#[derive(Clone, Copy)]
pub struct AlterEntry<C: Chromosome> {
    pub name: &'static str,
    pub convert: AlterConv<C>,
}

pub struct AlterRegistry<C: Chromosome + 'static> {
    entries: &'static [AlterEntry<C>],
}

impl<C> AlterRegistry<C>
where
    C: Chromosome + Clone + 'static,
{
    pub const fn new(entries: &'static [AlterEntry<C>]) -> Self {
        Self { entries }
    }

    pub fn convert(&self, input: &PyEngineInput) -> RadiateResult<Alterer<C>> {
        let entry = self
            .entries
            .iter()
            .find(|entry| entry.name == input.component.as_deref().unwrap_or(""))
            .ok_or_else(|| {
                let valid = self
                    .entries
                    .iter()
                    .map(|entry| entry.name)
                    .collect::<Vec<_>>()
                    .join(", ");

                radiate_err!(Builder: format!(
                    "Invalid alterer type '{:?}'. Valid alterers: [{}]",
                    input.component, valid
                ))
            })?;

        (entry.convert)(input)
    }
}

macro_rules! alter_table {
    ($($name:expr => $fn:ident),* $(,)?) => {
        &[
            $(
                AlterEntry {
                    name: $name,
                    convert: |input| {
                        $fn(input).map(|value| value.alterer())
                    },
                },
            )*
        ]
    };
}

macro_rules! impl_input_transform_for {
    ($chrom:ty, $registry_fn:ident) => {
        impl InputTransform<RadiateResult<Vec<Alterer<$chrom>>>> for PyEngineInput {
            fn transform(&self) -> RadiateResult<Vec<Alterer<$chrom>>> {
                alters_from_registry(self, $registry_fn())
            }
        }
    };
}

impl<C> InputTransform<RadiateResult<Vec<Alterer<C>>>> for &[PyEngineInput]
where
    C: Chromosome + Clone,
    PyEngineInput: InputTransform<RadiateResult<Vec<Alterer<C>>>>,
{
    fn transform(&self) -> RadiateResult<Vec<Alterer<C>>> {
        let mut alters = Vec::new();

        for input in self.iter() {
            if input.input_type != PyEngineInputType::Alterer {
                return Err(radiate_err!(Builder: format!(
                    "Input type {:?} is not an alterer",
                    input.input_type
                )));
            }

            alters.extend(input.transform()?);
        }

        Ok(alters)
    }
}

fn alters_from_registry<C>(
    input: &PyEngineInput,
    registry: AlterRegistry<C>,
) -> RadiateResult<Vec<Alterer<C>>>
where
    C: Chromosome + Clone + 'static,
{
    if input.input_type != PyEngineInputType::Alterer {
        return Err(radiate_err!(Builder: format!(
            "Input type {:?} is not an alterer",
            input.input_type
        )));
    }

    Ok(vec![registry.convert(input)?])
}

/// ---------------------------------------------------------------------------
/// INT REGISTRY
/// ---------------------------------------------------------------------------
fn int_registry<I>() -> AlterRegistry<IntChromosome<I>>
where
    I: Integer,
{
    AlterRegistry::new(alter_table! {
        crate::constants::components::MULTI_POINT_CROSSOVER   => convert_multi_point_crossover,
        crate::constants::components::UNIFORM_CROSSOVER       => convert_uniform_crossover,
        crate::constants::components::MEAN_CROSSOVER          => convert_mean_crossover,
        crate::constants::components::SHUFFLE_CROSSOVER       => convert_shuffle_crossover,

        crate::constants::components::ARITHMETIC_MUTATOR      => convert_arithmetic_mutator,
        crate::constants::components::SWAP_MUTATOR            => convert_swap_mutator,
        crate::constants::components::SCRAMBLE_MUTATOR        => convert_scramble_mutator,
        crate::constants::components::UNIFORM_MUTATOR         => convert_uniform_mutator,
        crate::constants::components::INVERSION_MUTATOR       => convert_inversion_mutator,
    })
}

/// ---------------------------------------------------------------------------
/// FLOAT REGISTRY
/// ---------------------------------------------------------------------------
fn float_registry<F>() -> AlterRegistry<FloatChromosome<F>>
where
    F: Float + NumericAllele,
{
    AlterRegistry::new(alter_table! {
        crate::constants::components::MULTI_POINT_CROSSOVER        => convert_multi_point_crossover,
        crate::constants::components::UNIFORM_CROSSOVER            => convert_uniform_crossover,
        crate::constants::components::MEAN_CROSSOVER               => convert_mean_crossover,
        crate::constants::components::INTERMEDIATE_CROSSOVER       => convert_intermediate_crossover,
        crate::constants::components::BLEND_CROSSOVER              => convert_blend_crossover,
        crate::constants::components::SIMULATED_BINARY_CROSSOVER   => convert_simulated_binary_crossover,

        crate::constants::components::GAUSSIAN_MUTATOR             => convert_gaussian_mutator,
        crate::constants::components::ARITHMETIC_MUTATOR           => convert_arithmetic_mutator,
        crate::constants::components::SWAP_MUTATOR                 => convert_swap_mutator,
        crate::constants::components::SCRAMBLE_MUTATOR             => convert_scramble_mutator,
        crate::constants::components::UNIFORM_MUTATOR              => convert_uniform_mutator,
        crate::constants::components::INVERSION_MUTATOR            => convert_inversion_mutator,
        crate::constants::components::POLYNOMIAL_MUTATOR           => convert_polynomial_mutator,
        crate::constants::components::JITTER_MUTATOR               => convert_jitter_mutator,
    })
}

/// ---------------------------------------------------------------------------
/// CHAR REGISTRY
/// ---------------------------------------------------------------------------
fn char_registry() -> AlterRegistry<CharChromosome> {
    AlterRegistry::new(alter_table! {
        crate::constants::components::MULTI_POINT_CROSSOVER   => convert_multi_point_crossover,
        crate::constants::components::UNIFORM_CROSSOVER       => convert_uniform_crossover,
        crate::constants::components::SHUFFLE_CROSSOVER       => convert_shuffle_crossover,

        crate::constants::components::SWAP_MUTATOR            => convert_swap_mutator,
        crate::constants::components::SCRAMBLE_MUTATOR        => convert_scramble_mutator,
        crate::constants::components::UNIFORM_MUTATOR         => convert_uniform_mutator,
        crate::constants::components::INVERSION_MUTATOR       => convert_inversion_mutator,
    })
}

/// ---------------------------------------------------------------------------
/// BIT REGISTRY
/// ---------------------------------------------------------------------------
fn bit_registry() -> AlterRegistry<BitChromosome> {
    AlterRegistry::new(alter_table! {
        crate::constants::components::MULTI_POINT_CROSSOVER   => convert_multi_point_crossover,
        crate::constants::components::UNIFORM_CROSSOVER       => convert_uniform_crossover,
        crate::constants::components::SHUFFLE_CROSSOVER       => convert_shuffle_crossover,

        crate::constants::components::SWAP_MUTATOR            => convert_swap_mutator,
        crate::constants::components::SCRAMBLE_MUTATOR        => convert_scramble_mutator,
        crate::constants::components::UNIFORM_MUTATOR         => convert_uniform_mutator,
        crate::constants::components::INVERSION_MUTATOR       => convert_inversion_mutator,
    })
}

/// ---------------------------------------------------------------------------
/// PERMUTATION REGISTRY
/// ---------------------------------------------------------------------------
fn perm_registry() -> AlterRegistry<PermutationChromosome<usize>> {
    AlterRegistry::new(alter_table! {
        crate::constants::components::PARTIALLY_MAPPED_CROSSOVER => convert_partially_mapped_crossover,
        crate::constants::components::EDGE_RECOMBINE_CROSSOVER   => convert_edge_recombine_crossover,

        crate::constants::components::SWAP_MUTATOR               => convert_swap_mutator,
        crate::constants::components::SCRAMBLE_MUTATOR           => convert_scramble_mutator,
        crate::constants::components::UNIFORM_MUTATOR            => convert_uniform_mutator,
        crate::constants::components::INVERSION_MUTATOR          => convert_inversion_mutator,
    })
}

/// ---------------------------------------------------------------------------
/// GRAPH REGISTRY
/// ---------------------------------------------------------------------------
fn graph_registry<F: OpFloat>() -> AlterRegistry<GraphChromosome<Op<F>>> {
    AlterRegistry::new(alter_table! {
        crate::constants::components::GRAPH_CROSSOVER       => convert_graph_crossover,

        crate::constants::components::GRAPH_MUTATOR         => convert_graph_mutator,
        crate::constants::components::OPERATION_MUTATOR     => convert_operation_mutator,
    })
}

/// ---------------------------------------------------------------------------
/// TREE REGISTRY
/// ---------------------------------------------------------------------------
fn tree_registry<F: OpFloat>() -> AlterRegistry<TreeChromosome<Op<F>>> {
    AlterRegistry::new(alter_table! {
        crate::constants::components::TREE_CROSSOVER        => convert_tree_crossover,

        crate::constants::components::HOIST_MUTATOR         => convert_hoist_mutator,
        crate::constants::components::OPERATION_MUTATOR     => convert_operation_mutator,
    })
}

impl_input_transform_for!(IntChromosome<u8>, int_registry);
impl_input_transform_for!(IntChromosome<u16>, int_registry);
impl_input_transform_for!(IntChromosome<u32>, int_registry);
impl_input_transform_for!(IntChromosome<u64>, int_registry);

impl_input_transform_for!(IntChromosome<i8>, int_registry);
impl_input_transform_for!(IntChromosome<i16>, int_registry);
impl_input_transform_for!(IntChromosome<i32>, int_registry);
impl_input_transform_for!(IntChromosome<i64>, int_registry);

impl_input_transform_for!(FloatChromosome<f32>, float_registry);
impl_input_transform_for!(FloatChromosome<f64>, float_registry);

impl_input_transform_for!(CharChromosome, char_registry);
impl_input_transform_for!(BitChromosome, bit_registry);
impl_input_transform_for!(PermutationChromosome<usize>, perm_registry);

impl_input_transform_for!(GraphChromosome<Op<f32>>, graph_registry);
impl_input_transform_for!(GraphChromosome<Op<f64>>, graph_registry);

impl_input_transform_for!(TreeChromosome<Op<f32>>, tree_registry);
impl_input_transform_for!(TreeChromosome<Op<f64>>, tree_registry);

/// ---------------------------------------------------------------------------
/// Concrete converters
/// ---------------------------------------------------------------------------
fn convert_jitter_mutator(input: &PyEngineInput) -> RadiateResult<JitterMutator> {
    let rate = input.extract::<PyExpr>("rate")?.inner;
    let magnitude = input.extract::<f64>("magnitude")?;

    Ok(JitterMutator::new(rate, magnitude as f32))
}

fn convert_inversion_mutator(input: &PyEngineInput) -> RadiateResult<InversionMutator> {
    let rate = input.extract::<PyExpr>("rate")?.inner;
    Ok(InversionMutator::new(rate))
}

fn convert_hoist_mutator(input: &PyEngineInput) -> RadiateResult<HoistMutator> {
    let rate = input.extract::<PyExpr>("rate")?.inner;
    Ok(HoistMutator::new(rate))
}

fn convert_tree_crossover(input: &PyEngineInput) -> RadiateResult<TreeCrossover> {
    let rate = input.extract::<PyExpr>("rate")?.inner;
    let max_size = input.extract::<i64>("max_size")?;

    Ok(TreeCrossover::new(rate).with_max_size(max_size as usize))
}

fn convert_multi_point_crossover(input: &PyEngineInput) -> RadiateResult<MultiPointCrossover> {
    let rate = input.extract::<PyExpr>("rate")?.inner;
    let points = input.extract::<i64>("num_points")?;

    Ok(MultiPointCrossover::new(rate, points as usize))
}

fn convert_uniform_crossover(input: &PyEngineInput) -> RadiateResult<UniformCrossover> {
    let rate = input.extract::<PyExpr>("rate")?.inner;
    Ok(UniformCrossover::new(rate))
}

fn convert_uniform_mutator(input: &PyEngineInput) -> RadiateResult<UniformMutator> {
    let rate = input.extract::<PyExpr>("rate")?.inner;
    Ok(UniformMutator::new(rate))
}

fn convert_mean_crossover(input: &PyEngineInput) -> RadiateResult<MeanCrossover> {
    let rate = input.extract::<PyExpr>("rate")?.inner;
    Ok(MeanCrossover::new(rate))
}

fn convert_intermediate_crossover(input: &PyEngineInput) -> RadiateResult<IntermediateCrossover> {
    let rate = input.extract::<PyExpr>("rate")?.inner;
    let alpha = input.extract::<f64>("alpha")?;

    Ok(IntermediateCrossover::new(rate, alpha as f32))
}

fn convert_blend_crossover(input: &PyEngineInput) -> RadiateResult<BlendCrossover> {
    let rate = input.extract::<PyExpr>("rate")?.inner;
    let alpha = input.extract::<f64>("alpha")?;

    Ok(BlendCrossover::new(rate, alpha as f32))
}

fn convert_shuffle_crossover(input: &PyEngineInput) -> RadiateResult<ShuffleCrossover> {
    let rate = input.extract::<PyExpr>("rate")?.inner;
    Ok(ShuffleCrossover::new(rate))
}

fn convert_partially_mapped_crossover(input: &PyEngineInput) -> RadiateResult<PMXCrossover> {
    let rate = input.extract::<PyExpr>("rate")?.inner;
    Ok(PMXCrossover::new(rate))
}

fn convert_scramble_mutator(input: &PyEngineInput) -> RadiateResult<ScrambleMutator> {
    let rate = input.extract::<PyExpr>("rate")?.inner;
    Ok(ScrambleMutator::new(rate))
}

fn convert_swap_mutator(input: &PyEngineInput) -> RadiateResult<SwapMutator> {
    let rate = input.extract::<PyExpr>("rate")?.inner;
    Ok(SwapMutator::new(rate))
}

fn convert_arithmetic_mutator(input: &PyEngineInput) -> RadiateResult<ArithmeticMutator> {
    let rate = input.extract::<PyExpr>("rate")?.inner;
    Ok(ArithmeticMutator::new(rate))
}

fn convert_gaussian_mutator(input: &PyEngineInput) -> RadiateResult<GaussianMutator> {
    let rate = input.extract::<PyExpr>("rate")?.inner;
    Ok(GaussianMutator::new(rate))
}

fn convert_simulated_binary_crossover(
    input: &PyEngineInput,
) -> RadiateResult<SimulatedBinaryCrossover> {
    let rate = input.extract::<PyExpr>("rate")?.inner;
    let contiguity = input.extract::<f64>("contiguity")?;

    Ok(SimulatedBinaryCrossover::new(rate, contiguity as f32))
}

fn convert_graph_crossover(input: &PyEngineInput) -> RadiateResult<GraphCrossover> {
    let rate = input.extract::<PyExpr>("rate")?.inner;
    let parent_node_rate = input.extract::<PyExpr>("parent_node_rate")?.inner;

    Ok(GraphCrossover::new(rate, parent_node_rate))
}

fn convert_graph_mutator(input: &PyEngineInput) -> RadiateResult<GraphMutator> {
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

fn convert_operation_mutator(input: &PyEngineInput) -> RadiateResult<OperationMutator> {
    let rate = input.extract::<PyExpr>("rate")?.inner;
    let replace_rate = input.extract::<PyExpr>("replace_rate")?.inner;

    Ok(OperationMutator::new(rate, replace_rate))
}

fn convert_edge_recombine_crossover(
    input: &PyEngineInput,
) -> RadiateResult<EdgeRecombinationCrossover> {
    let rate = input.extract::<PyExpr>("rate")?.inner;

    Ok(EdgeRecombinationCrossover::new(rate))
}

fn convert_polynomial_mutator(input: &PyEngineInput) -> RadiateResult<PolynomialMutator> {
    let rate = input.extract::<PyExpr>("rate")?.inner;
    let eta = input.extract::<f64>("eta")?;

    Ok(PolynomialMutator::new(rate, eta as f32))
}
