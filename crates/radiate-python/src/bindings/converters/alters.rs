use crate::{InputTransform, PyEngineInput, PyEngineInputType};
use pyo3::{PyResult, exceptions::PyTypeError};
use radiate::{chromosomes::NumericAllele, *};
use radiate_utils::{Float, Integer};
use std::collections::HashMap;

type AlterConv<C> = fn(&PyEngineInput) -> RadiateResult<Alterer<C>>;

macro_rules! table {
    ($($name:expr => $fn:ident),* $(,)?) => {{
        use std::collections::HashMap;
        let mut m: HashMap<&'static str, AlterConv<_>> = HashMap::new();
        $(
            m.insert($name, |inp| {
                $fn(inp).map(|val| val.alterer())
            });
        )*
        m
    }};
}

macro_rules! impl_input_transform_for {
    ($chrom:ty, $map_fn:ident) => {
        impl InputTransform<Vec<Alterer<$chrom>>> for PyEngineInput {
            fn transform(&self) -> Vec<Alterer<$chrom>> {
                alters_from_table(self, $map_fn())
                    .expect("Failed to convert alterer from PyEngineInput")
                    .into()
            }
        }
    };
}

impl_input_transform_for!(IntChromosome<u8>, int_alterers);
impl_input_transform_for!(IntChromosome<u16>, int_alterers);
impl_input_transform_for!(IntChromosome<u32>, int_alterers);
impl_input_transform_for!(IntChromosome<u64>, int_alterers);

impl_input_transform_for!(IntChromosome<i8>, int_alterers);
impl_input_transform_for!(IntChromosome<i16>, int_alterers);
impl_input_transform_for!(IntChromosome<i32>, int_alterers);
impl_input_transform_for!(IntChromosome<i64>, int_alterers);

impl_input_transform_for!(FloatChromosome<f32>, float_alterers);
impl_input_transform_for!(FloatChromosome<f64>, float_alterers);

impl_input_transform_for!(CharChromosome, char_alterers);
impl_input_transform_for!(BitChromosome, bit_alterers);
impl_input_transform_for!(PermutationChromosome<usize>, perm_alterers);
impl_input_transform_for!(GraphChromosome<Op<f32>>, graph_alterers);
impl_input_transform_for!(TreeChromosome<Op<f32>>, tree_alterers);

impl<C> InputTransform<Vec<Alterer<C>>> for &[PyEngineInput]
where
    C: Chromosome + Clone,
    PyEngineInput: InputTransform<Vec<Alterer<C>>>,
{
    fn transform(&self) -> Vec<Alterer<C>> {
        let mut alters: Vec<Alterer<C>> = Vec::new();

        for input in self.iter() {
            if input.input_type != PyEngineInputType::Alterer {
                panic!("Input type {:?} not an alterer", input.input_type);
            }

            let mut converted = input.transform();
            alters.append(&mut converted);
        }

        alters
    }
}

fn alters_from_table<C>(
    input: &PyEngineInput,
    table: HashMap<&'static str, AlterConv<C>>,
) -> PyResult<Vec<Alterer<C>>>
where
    C: Chromosome + Clone + 'static,
{
    if input.input_type != PyEngineInputType::Alterer {
        return Err(PyTypeError::new_err(format!(
            "Input type {:?} is not an alterer",
            input.input_type
        )));
    }

    if let Some(make) = table.get(input.component.as_str()) {
        Ok(vec![make(input)?])
    } else {
        Err(PyTypeError::new_err(format!(
            "Invalid alterer type: {}",
            input.component
        )))
    }
}

// INT
fn int_alterers<I: Integer>() -> HashMap<&'static str, AlterConv<IntChromosome<I>>> {
    table! {
        crate::names::MULTI_POINT_CROSSOVER   => convert_multi_point_crossover,
        crate::names::UNIFORM_CROSSOVER       => convert_uniform_crossover,
        crate::names::MEAN_CROSSOVER          => convert_mean_crossover,
        crate::names::SHUFFLE_CROSSOVER       => convert_shuffle_crossover,

        crate::names::ARITHMETIC_MUTATOR      => convert_arithmetic_mutator,
        crate::names::SWAP_MUTATOR            => convert_swap_mutator,
        crate::names::SCRAMBLE_MUTATOR        => convert_scramble_mutator,
        crate::names::UNIFORM_MUTATOR         => convert_uniform_mutator,
        crate::names::INVERSION_MUTATOR       => convert_inversion_mutator,
    }
}

// FLOAT
fn float_alterers<F: Float + NumericAllele>() -> HashMap<&'static str, AlterConv<FloatChromosome<F>>>
{
    table! {
        crate::names::MULTI_POINT_CROSSOVER        => convert_multi_point_crossover,
        crate::names::UNIFORM_CROSSOVER            => convert_uniform_crossover,
        crate::names::MEAN_CROSSOVER               => convert_mean_crossover,
        crate::names::INTERMEDIATE_CROSSOVER       => convert_intermediate_crossover,
        crate::names::BLEND_CROSSOVER              => convert_blend_crossover,
        crate::names::SIMULATED_BINARY_CROSSOVER   => convert_simulated_binary_crossover,

        crate::names::GAUSSIAN_MUTATOR             => convert_gaussian_mutator,
        crate::names::ARITHMETIC_MUTATOR           => convert_arithmetic_mutator,
        crate::names::SWAP_MUTATOR                 => convert_swap_mutator,
        crate::names::SCRAMBLE_MUTATOR             => convert_scramble_mutator,
        crate::names::UNIFORM_MUTATOR              => convert_uniform_mutator,
        crate::names::INVERSION_MUTATOR            => convert_inversion_mutator,
        crate::names::POLYNOMIAL_MUTATOR           => convert_polynomial_mutator,
        crate::names::JITTER_MUTATOR               => convert_jitter_mutator,
    }
}

// CHAR
fn char_alterers() -> HashMap<&'static str, AlterConv<CharChromosome>> {
    table! {
        crate::names::MULTI_POINT_CROSSOVER   => convert_multi_point_crossover,
        crate::names::UNIFORM_CROSSOVER       => convert_uniform_crossover,
        crate::names::SHUFFLE_CROSSOVER       => convert_shuffle_crossover,

        crate::names::SWAP_MUTATOR            => convert_swap_mutator,
        crate::names::SCRAMBLE_MUTATOR        => convert_scramble_mutator,
        crate::names::UNIFORM_MUTATOR         => convert_uniform_mutator,
        crate::names::INVERSION_MUTATOR       => convert_inversion_mutator,
    }
}

// BIT
fn bit_alterers() -> HashMap<&'static str, AlterConv<BitChromosome>> {
    table! {
        crate::names::MULTI_POINT_CROSSOVER   => convert_multi_point_crossover,
        crate::names::UNIFORM_CROSSOVER       => convert_uniform_crossover,
        crate::names::SHUFFLE_CROSSOVER       => convert_shuffle_crossover,

        crate::names::SWAP_MUTATOR            => convert_swap_mutator,
        crate::names::SCRAMBLE_MUTATOR        => convert_scramble_mutator,
        crate::names::UNIFORM_MUTATOR         => convert_uniform_mutator,
        crate::names::INVERSION_MUTATOR       => convert_inversion_mutator,
    }
}

// PERMUTATION<usize>
fn perm_alterers() -> HashMap<&'static str, AlterConv<PermutationChromosome<usize>>> {
    table! {
        crate::names::PARTIALLY_MAPPED_CROSSOVER  => convert_partially_mapped_crossover,
        crate::names::EDGE_RECOMBINE_CROSSOVER    => convert_edge_recombine_crossover,

        crate::names::SWAP_MUTATOR                => convert_swap_mutator,
        crate::names::SCRAMBLE_MUTATOR            => convert_scramble_mutator,
        crate::names::UNIFORM_MUTATOR             => convert_uniform_mutator,
        crate::names::INVERSION_MUTATOR           => convert_inversion_mutator,
    }
}

// GRAPH<Op<f32>>
fn graph_alterers() -> HashMap<&'static str, AlterConv<GraphChromosome<Op<f32>>>> {
    table! {
        crate::names::GRAPH_CROSSOVER       => convert_graph_crossover,

        crate::names::GRAPH_MUTATOR         => convert_graph_mutator,
        crate::names::OPERATION_MUTATOR     => convert_operation_mutator,
    }
}

// TREE<Op<f32>>
fn tree_alterers() -> HashMap<&'static str, AlterConv<TreeChromosome<Op<f32>>>> {
    table! {
        crate::names::TREE_CROSSOVER        => convert_tree_crossover,

        crate::names::HOIST_MUTATOR         => convert_hoist_mutator,
        crate::names::OPERATION_MUTATOR     => convert_operation_mutator,
    }
}

/// Concrete alterer conversion functions
/// These functions take a PyEngineInput and extract parameters to create the corresponding alterer.
/// Each function corresponds to a specific alterer type.
///
/// Because python is dynamically typed, there is no guarantee that the parameters exist or are of the correct type.
/// We do our best above to ensure that the correct parameters are provided by relying
/// as much as possible on rust's type system, but we provide default values where appropriate.
/// If a parameter is missing, we use a sensible default.
/// -------------------------------------------------------------------
fn convert_jitter_mutator(input: &PyEngineInput) -> RadiateResult<JitterMutator> {
    let rate = input.get_rate().unwrap();
    let magnitude = input.extract::<f64>("magnitude")?;

    Ok(JitterMutator::new(rate, magnitude as f32))
}

fn convert_inversion_mutator(input: &PyEngineInput) -> RadiateResult<InversionMutator> {
    let rate = input.get_rate().unwrap();
    Ok(InversionMutator::new(rate))
}

fn convert_hoist_mutator(input: &PyEngineInput) -> RadiateResult<HoistMutator> {
    let rate = input.get_rate().unwrap();
    Ok(HoistMutator::new(rate))
}

fn convert_tree_crossover(input: &PyEngineInput) -> RadiateResult<TreeCrossover> {
    let rate = input.get_rate().unwrap();
    let max_size = input.extract::<i64>("max_size")?;
    Ok(TreeCrossover::new(rate).with_max_size(max_size as usize))
}

fn convert_multi_point_crossover(input: &PyEngineInput) -> RadiateResult<MultiPointCrossover> {
    let rate = input.get_rate().unwrap();
    let points = input.extract::<i64>("num_points")?;

    Ok(MultiPointCrossover::new(rate, points as usize))
}

fn convert_uniform_crossover(input: &PyEngineInput) -> RadiateResult<UniformCrossover> {
    let rate = input.get_rate().unwrap();
    Ok(UniformCrossover::new(rate))
}

fn convert_uniform_mutator(input: &PyEngineInput) -> RadiateResult<UniformMutator> {
    let rate = input.get_rate().unwrap();
    Ok(UniformMutator::new(rate))
}

fn convert_mean_crossover(input: &PyEngineInput) -> RadiateResult<MeanCrossover> {
    let rate = input.get_rate().unwrap();
    Ok(MeanCrossover::new(rate))
}

fn convert_intermediate_crossover(input: &PyEngineInput) -> RadiateResult<IntermediateCrossover> {
    let rate = input.get_rate().unwrap();
    let alpha = input.extract::<f64>("alpha")?;
    Ok(IntermediateCrossover::new(rate, alpha as f32))
}

fn convert_blend_crossover(input: &PyEngineInput) -> RadiateResult<BlendCrossover> {
    let rate = input.get_rate().unwrap();
    let alpha = input.extract::<f64>("alpha")?;
    Ok(BlendCrossover::new(rate, alpha as f32))
}

fn convert_shuffle_crossover(input: &PyEngineInput) -> RadiateResult<ShuffleCrossover> {
    let rate = input.get_rate().unwrap();
    Ok(ShuffleCrossover::new(rate))
}

fn convert_partially_mapped_crossover(input: &PyEngineInput) -> RadiateResult<PMXCrossover> {
    let rate = input.get_rate().unwrap();
    Ok(PMXCrossover::new(rate))
}

fn convert_scramble_mutator(input: &PyEngineInput) -> RadiateResult<ScrambleMutator> {
    let rate = input.get_rate().unwrap();
    Ok(ScrambleMutator::new(rate))
}

fn convert_swap_mutator(input: &PyEngineInput) -> RadiateResult<SwapMutator> {
    let rate = input.get_rate().unwrap();
    Ok(SwapMutator::new(rate))
}

fn convert_arithmetic_mutator(input: &PyEngineInput) -> RadiateResult<ArithmeticMutator> {
    let rate = input.get_rate().unwrap();
    Ok(ArithmeticMutator::new(rate))
}

fn convert_gaussian_mutator(input: &PyEngineInput) -> RadiateResult<GaussianMutator> {
    let rate = input.get_rate().unwrap();

    Ok(GaussianMutator::new(rate))
}

fn convert_simulated_binary_crossover(
    input: &PyEngineInput,
) -> RadiateResult<SimulatedBinaryCrossover> {
    let rate = input.get_rate().unwrap();
    let contiguity = input.extract::<f64>("contiguity")?;
    Ok(SimulatedBinaryCrossover::new(rate, contiguity as f32))
}

fn convert_graph_crossover(input: &PyEngineInput) -> RadiateResult<GraphCrossover> {
    let rate = input.get_rate().unwrap();
    let parent_node_rate = input.extract::<f64>("parent_node_rate")?;

    Ok(GraphCrossover::new(rate, parent_node_rate as f32))
}

fn convert_graph_mutator(input: &PyEngineInput) -> RadiateResult<GraphMutator> {
    let vertex_rate = input.extract::<f64>("vertex_rate")?;
    let edge_rate = input.extract::<f64>("edge_rate")?;
    let allow_recurrent = input.extract::<bool>("allow_recurrent")?;

    Ok(GraphMutator::new(vertex_rate as f32, edge_rate as f32).allow_recurrent(allow_recurrent))
}

fn convert_operation_mutator(input: &PyEngineInput) -> RadiateResult<OperationMutator> {
    let rate = input.get_rate().unwrap();
    let replace_rate = input.extract::<f64>("replace_rate")?;

    Ok(OperationMutator::new(rate, replace_rate as f32))
}

fn convert_edge_recombine_crossover(
    input: &PyEngineInput,
) -> RadiateResult<EdgeRecombinationCrossover> {
    let rate = input.get_rate().unwrap();
    Ok(EdgeRecombinationCrossover::new(rate))
}

fn convert_polynomial_mutator(input: &PyEngineInput) -> RadiateResult<PolynomialMutator> {
    let rate = input.get_rate().unwrap();
    let eta = input.extract::<f64>("eta")?;
    Ok(PolynomialMutator::new(rate, eta as f32))
}
