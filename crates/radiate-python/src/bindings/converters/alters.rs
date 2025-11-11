use crate::{AnyChromosome, InputTransform, PyEngineInput, PyEngineInputType};
use pyo3::{PyResult, exceptions::PyTypeError};
use radiate::*;
use std::collections::HashMap;

type AlterConv<C> = fn(&PyEngineInput) -> Box<dyn Alter<C>>;

macro_rules! table {
    ($($name:expr => $fn:ident),* $(,)?) => {{
        use std::collections::HashMap;
        let mut m: HashMap<&'static str, AlterConv<_>> = HashMap::new();
        $(
            m.insert($name, |inp| {
                // $fn returns a concrete alterer type that implements Alter<C>
                // We box that concrete type and cast to the trait object.
                Box::new($fn(inp).alterer())
            });
        )*
        m
    }};
}

macro_rules! impl_input_transform_for {
    ($chrom:ty, $map_fn:ident) => {
        impl InputTransform<Vec<Box<dyn Alter<$chrom>>>> for PyEngineInput {
            fn transform(&self) -> Vec<Box<dyn Alter<$chrom>>> {
                alters_from_table(self, $map_fn())
                    .expect("alter conversion")
                    .into()
            }
        }
    };
}

impl_input_transform_for!(IntChromosome<i32>, int_alterers);
impl_input_transform_for!(FloatChromosome, float_alterers);
impl_input_transform_for!(CharChromosome, char_alterers);
impl_input_transform_for!(BitChromosome, bit_alterers);
impl_input_transform_for!(PermutationChromosome<usize>, perm_alterers);
impl_input_transform_for!(GraphChromosome<Op<f32>>, graph_alterers);
impl_input_transform_for!(TreeChromosome<Op<f32>>, tree_alterers);
impl_input_transform_for!(AnyChromosome<'static>, any_alterers);

impl<C> InputTransform<Vec<Box<dyn Alter<C>>>> for &[PyEngineInput]
where
    C: Chromosome + Clone,
    PyEngineInput: InputTransform<Vec<Box<dyn Alter<C>>>>,
{
    fn transform(&self) -> Vec<Box<dyn Alter<C>>> {
        let mut alters: Vec<Box<dyn Alter<C>>> = Vec::new();

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
    table: &HashMap<&'static str, AlterConv<C>>,
) -> PyResult<Vec<Box<dyn Alter<C>>>>
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
        Ok(vec![make(input)])
    } else {
        Err(PyTypeError::new_err(format!(
            "Invalid alterer type: {}",
            input.component
        )))
    }
}

// INT
fn int_alterers() -> &'static HashMap<&'static str, AlterConv<IntChromosome<i32>>> {
    use std::sync::OnceLock;
    static MAP: OnceLock<HashMap<&'static str, AlterConv<IntChromosome<i32>>>> = OnceLock::new();
    MAP.get_or_init(|| {
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
    })
}

// FLOAT
fn float_alterers() -> &'static HashMap<&'static str, AlterConv<FloatChromosome>> {
    use std::sync::OnceLock;
    static MAP: OnceLock<HashMap<&'static str, AlterConv<FloatChromosome>>> = OnceLock::new();
    MAP.get_or_init(|| {
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
    })
}

// CHAR
fn char_alterers() -> &'static HashMap<&'static str, AlterConv<CharChromosome>> {
    use std::sync::OnceLock;
    static MAP: OnceLock<HashMap<&'static str, AlterConv<CharChromosome>>> = OnceLock::new();
    MAP.get_or_init(|| {
        table! {
            crate::names::MULTI_POINT_CROSSOVER   => convert_multi_point_crossover,
            crate::names::UNIFORM_CROSSOVER       => convert_uniform_crossover,
            crate::names::SHUFFLE_CROSSOVER       => convert_shuffle_crossover,
            crate::names::SWAP_MUTATOR            => convert_swap_mutator,
            crate::names::SCRAMBLE_MUTATOR        => convert_scramble_mutator,
            crate::names::UNIFORM_MUTATOR         => convert_uniform_mutator,
            crate::names::INVERSION_MUTATOR       => convert_inversion_mutator,
        }
    })
}

// BIT
fn bit_alterers() -> &'static HashMap<&'static str, AlterConv<BitChromosome>> {
    use std::sync::OnceLock;
    static MAP: OnceLock<HashMap<&'static str, AlterConv<BitChromosome>>> = OnceLock::new();
    MAP.get_or_init(|| {
        table! {
            crate::names::MULTI_POINT_CROSSOVER   => convert_multi_point_crossover,
            crate::names::UNIFORM_CROSSOVER       => convert_uniform_crossover,
            crate::names::SHUFFLE_CROSSOVER       => convert_shuffle_crossover,
            crate::names::SWAP_MUTATOR            => convert_swap_mutator,
            crate::names::SCRAMBLE_MUTATOR        => convert_scramble_mutator,
            crate::names::UNIFORM_MUTATOR         => convert_uniform_mutator,
            crate::names::INVERSION_MUTATOR       => convert_inversion_mutator,
        }
    })
}

// PERMUTATION<usize>
fn perm_alterers() -> &'static HashMap<&'static str, AlterConv<PermutationChromosome<usize>>> {
    use std::sync::OnceLock;
    static MAP: OnceLock<HashMap<&'static str, AlterConv<PermutationChromosome<usize>>>> =
        OnceLock::new();
    MAP.get_or_init(|| {
        table! {
            crate::names::PARTIALLY_MAPPED_CROSSOVER  => convert_partially_mapped_crossover,
            crate::names::EDGE_RECOMBINE_CROSSOVER    => convert_edge_recombine_crossover,
            crate::names::SWAP_MUTATOR                => convert_swap_mutator,
            crate::names::SCRAMBLE_MUTATOR            => convert_scramble_mutator,
            crate::names::UNIFORM_MUTATOR             => convert_uniform_mutator,
            crate::names::INVERSION_MUTATOR           => convert_inversion_mutator,
        }
    })
}

// GRAPH<Op<f32>>
fn graph_alterers() -> &'static HashMap<&'static str, AlterConv<GraphChromosome<Op<f32>>>> {
    use std::sync::OnceLock;
    static MAP: OnceLock<HashMap<&'static str, AlterConv<GraphChromosome<Op<f32>>>>> =
        OnceLock::new();
    MAP.get_or_init(|| {
        table! {
            crate::names::GRAPH_CROSSOVER       => convert_graph_crossover,
            crate::names::GRAPH_MUTATOR         => convert_graph_mutator,
            crate::names::OPERATION_MUTATOR     => convert_operation_mutator,
        }
    })
}

// TREE<Op<f32>>
fn tree_alterers() -> &'static HashMap<&'static str, AlterConv<TreeChromosome<Op<f32>>>> {
    use std::sync::OnceLock;
    static MAP: OnceLock<HashMap<&'static str, AlterConv<TreeChromosome<Op<f32>>>>> =
        OnceLock::new();
    MAP.get_or_init(|| {
        table! {
            crate::names::TREE_CROSSOVER        => convert_tree_crossover,
            crate::names::HOIST_MUTATOR         => convert_hoist_mutator,
            crate::names::OPERATION_MUTATOR     => convert_operation_mutator,
        }
    })
}

// ANY (generic bag of common alterers exposed for AnyChromosome)
fn any_alterers() -> &'static HashMap<&'static str, AlterConv<AnyChromosome<'static>>> {
    use std::sync::OnceLock;
    static MAP: OnceLock<HashMap<&'static str, AlterConv<AnyChromosome<'static>>>> =
        OnceLock::new();
    MAP.get_or_init(|| {
        table! {
            crate::names::MULTI_POINT_CROSSOVER   => convert_multi_point_crossover,
            crate::names::UNIFORM_CROSSOVER       => convert_uniform_crossover,
            crate::names::SHUFFLE_CROSSOVER       => convert_shuffle_crossover,
            crate::names::MEAN_CROSSOVER          => convert_mean_crossover,
            crate::names::SWAP_MUTATOR            => convert_swap_mutator,
            crate::names::SCRAMBLE_MUTATOR        => convert_scramble_mutator,
            crate::names::UNIFORM_MUTATOR         => convert_uniform_mutator,
            crate::names::INVERSION_MUTATOR       => convert_inversion_mutator,
            crate::names::ARITHMETIC_MUTATOR      => convert_arithmetic_mutator,
        }
    })
}

fn convert_jitter_mutator(input: &PyEngineInput) -> JitterMutator {
    let rate = input.get_f32("rate").unwrap_or(0.5);
    let magnitude = input.get_f32("magnitude").unwrap_or(0.5);
    JitterMutator::new(rate, magnitude)
}

fn convert_inversion_mutator(input: &PyEngineInput) -> InversionMutator {
    let rate = input.get_f32("rate").unwrap_or(0.5);
    InversionMutator::new(rate)
}

fn convert_hoist_mutator(input: &PyEngineInput) -> HoistMutator {
    let rate = input.get_f32("rate").unwrap_or(0.5);
    HoistMutator::new(rate)
}

fn convert_tree_crossover(input: &PyEngineInput) -> TreeCrossover {
    let rate = input.get_f32("rate").unwrap_or(0.5);
    let max_size = input.get_usize("max_size").unwrap_or(30);
    TreeCrossover::new(rate).with_max_size(max_size)
}

fn convert_multi_point_crossover(input: &PyEngineInput) -> MultiPointCrossover {
    let rate = input.get_f32("rate").unwrap_or(0.5);
    let points = input.get_usize("num_points").unwrap_or(2);

    MultiPointCrossover::new(rate, points)
}

fn convert_uniform_crossover(input: &PyEngineInput) -> UniformCrossover {
    let rate = input.get_f32("rate").unwrap_or(0.5);
    UniformCrossover::new(rate)
}

fn convert_uniform_mutator(input: &PyEngineInput) -> UniformMutator {
    let rate = input.get_f32("rate").unwrap_or(0.5);
    UniformMutator::new(rate)
}

fn convert_mean_crossover(input: &PyEngineInput) -> MeanCrossover {
    let rate = input.get_f32("rate").unwrap_or(0.5);
    MeanCrossover::new(rate)
}

fn convert_intermediate_crossover(input: &PyEngineInput) -> IntermediateCrossover {
    let rate = input.get_f32("rate").unwrap_or(0.5);
    let alpha = input.get_f32("alpha").unwrap_or(0.5);
    IntermediateCrossover::new(rate, alpha)
}

fn convert_blend_crossover(input: &PyEngineInput) -> BlendCrossover {
    let rate = input.get_f32("rate").unwrap_or(0.5);
    let alpha = input.get_f32("alpha").unwrap_or(0.5);
    BlendCrossover::new(rate, alpha)
}

fn convert_shuffle_crossover(input: &PyEngineInput) -> ShuffleCrossover {
    let rate = input.get_f32("rate").unwrap_or(0.5);
    ShuffleCrossover::new(rate)
}

fn convert_partially_mapped_crossover(input: &PyEngineInput) -> PMXCrossover {
    let rate = input.get_f32("rate").unwrap_or(0.5);
    PMXCrossover::new(rate)
}

fn convert_scramble_mutator(input: &PyEngineInput) -> ScrambleMutator {
    let rate = input.get_f32("rate").unwrap_or(0.5);
    ScrambleMutator::new(rate)
}

fn convert_swap_mutator(input: &PyEngineInput) -> SwapMutator {
    let rate = input.get_f32("rate").unwrap_or(0.5);
    SwapMutator::new(rate)
}

fn convert_arithmetic_mutator(input: &PyEngineInput) -> ArithmeticMutator {
    let rate = input.get_f32("rate").unwrap_or(0.5);
    ArithmeticMutator::new(rate)
}

fn convert_gaussian_mutator(input: &PyEngineInput) -> GaussianMutator {
    let rate = input.get_f32("rate").unwrap_or(0.5);

    GaussianMutator::new(rate)
}

fn convert_simulated_binary_crossover(input: &PyEngineInput) -> SimulatedBinaryCrossover {
    let rate = input.get_f32("rate").unwrap_or(0.5);
    let contiguity = input.get_f32("contiguity").unwrap_or(0.5);
    SimulatedBinaryCrossover::new(rate, contiguity)
}

fn convert_graph_crossover(input: &PyEngineInput) -> GraphCrossover {
    let rate = input.get_f32("rate").unwrap_or(0.5);
    let parent_node_rate = input.get_f32("parent_node_rate").unwrap_or(0.5);

    GraphCrossover::new(rate, parent_node_rate)
}

fn convert_graph_mutator(input: &PyEngineInput) -> GraphMutator {
    let vertex_rate = input.get_f32("vertex_rate").unwrap_or(0.5);
    let edge_rate = input.get_f32("edge_rate").unwrap_or(0.5);
    let allow_recurrent = input.get_bool("allow_recurrent").unwrap_or(false);

    GraphMutator::new(vertex_rate, edge_rate).allow_recurrent(allow_recurrent)
}

fn convert_operation_mutator(input: &PyEngineInput) -> OperationMutator {
    let rate = input.get_f32("rate").unwrap_or(0.5);
    let replace_rate = input.get_f32("replace_rate").unwrap_or(0.5);

    OperationMutator::new(rate, replace_rate)
}

fn convert_edge_recombine_crossover(input: &PyEngineInput) -> EdgeRecombinationCrossover {
    let rate = input.get_f32("rate").unwrap_or(0.5);
    EdgeRecombinationCrossover::new(rate)
}

fn convert_polynomial_mutator(input: &PyEngineInput) -> PolynomialMutator {
    let rate = input.get_f32("rate").unwrap_or(0.5);
    let eta = input.get_f32("eta").unwrap_or(20.0);
    PolynomialMutator::new(rate, eta)
}
