use crate::{
    AnyChromosome, InputTransform, PyChromosome, PyCrossover, PyEngineInput, PyEngineInputType,
    PyMutator,
};
use pyo3::{Py, PyAny};
use radiate::*;

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

impl InputTransform<Vec<Box<dyn Alter<IntChromosome<i32>>>>> for PyEngineInput {
    fn transform(&self) -> Vec<Box<dyn Alter<IntChromosome<i32>>>> {
        if self.input_type != PyEngineInputType::Alterer {
            panic!("Input type {:?} not an alterer", self.input_type);
        }

        match self.component.as_str() {
            crate::names::MULTI_POINT_CROSSOVER => alters!(convert_multi_point_crossover(&self)),
            crate::names::UNIFORM_CROSSOVER => alters!(convert_uniform_crossover(&self)),
            crate::names::MEAN_CROSSOVER => alters!(convert_mean_crossover(&self)),
            crate::names::SHUFFLE_CROSSOVER => alters!(convert_shuffle_crossover(&self)),
            crate::names::ARITHMETIC_MUTATOR => alters!(convert_arithmetic_mutator(&self)),
            crate::names::SWAP_MUTATOR => alters!(convert_swap_mutator(&self)),
            crate::names::SCRAMBLE_MUTATOR => alters!(convert_scramble_mutator(&self)),
            crate::names::GAUSSIAN_MUTATOR => alters!(convert_gaussian_mutator(&self)),
            crate::names::UNIFORM_MUTATOR => alters!(convert_uniform_mutator(&self)),
            crate::names::INVERSION_MUTATOR => alters!(convert_inversion_mutator(&self)),
            crate::names::CUSTOM_MUTATOR => alters!(convert_custom_mutator(&self)),
            crate::names::CUSTOM_CROSSOVER => alters!(convert_custom_crossover(&self)),
            _ => panic!("Invalid alterer type {}", self.component),
        }
    }
}

impl InputTransform<Vec<Box<dyn Alter<FloatChromosome>>>> for PyEngineInput {
    fn transform(&self) -> Vec<Box<dyn Alter<FloatChromosome>>> {
        if self.input_type != PyEngineInputType::Alterer {
            panic!("Input type {:?} not an alterer", self.input_type);
        }

        match self.component.as_str() {
            crate::names::MULTI_POINT_CROSSOVER => alters!(convert_multi_point_crossover(&self)),
            crate::names::UNIFORM_CROSSOVER => alters!(convert_uniform_crossover(&self)),
            crate::names::MEAN_CROSSOVER => alters!(convert_mean_crossover(&self)),
            crate::names::INTERMEDIATE_CROSSOVER => alters!(convert_intermediate_crossover(&self)),
            crate::names::BLEND_CROSSOVER => alters!(convert_blend_crossover(&self)),
            crate::names::CUSTOM_CROSSOVER => alters!(convert_custom_crossover(&self)),
            crate::names::SIMULATED_BINARY_CROSSOVER => {
                alters!(convert_simulated_binary_crossover(&self))
            }
            crate::names::GAUSSIAN_MUTATOR => alters!(convert_gaussian_mutator(&self)),
            crate::names::ARITHMETIC_MUTATOR => alters!(convert_arithmetic_mutator(&self)),
            crate::names::SWAP_MUTATOR => alters!(convert_swap_mutator(&self)),
            crate::names::SCRAMBLE_MUTATOR => alters!(convert_scramble_mutator(&self)),
            crate::names::UNIFORM_MUTATOR => alters!(convert_uniform_mutator(&self)),
            crate::names::INVERSION_MUTATOR => alters!(convert_inversion_mutator(&self)),
            crate::names::POLYNOMIAL_MUTATOR => alters!(convert_polynomial_mutator(&self)),
            crate::names::CUSTOM_MUTATOR => alters!(convert_custom_mutator(&self)),
            crate::names::JITTER_MUTATOR => alters!(convert_jitter_mutator(&self)),
            _ => panic!("Invalid alterer type {}", self.component),
        }
    }
}

impl InputTransform<Vec<Box<dyn Alter<CharChromosome>>>> for PyEngineInput {
    fn transform(&self) -> Vec<Box<dyn Alter<CharChromosome>>> {
        if self.input_type != PyEngineInputType::Alterer {
            panic!("Input type {:?} not an alterer", self.input_type);
        }

        match self.component.as_str() {
            crate::names::MULTI_POINT_CROSSOVER => alters!(convert_multi_point_crossover(&self)),
            crate::names::UNIFORM_CROSSOVER => alters!(convert_uniform_crossover(&self)),
            crate::names::SHUFFLE_CROSSOVER => alters!(convert_shuffle_crossover(&self)),
            crate::names::CUSTOM_CROSSOVER => alters!(convert_custom_crossover(&self)),
            crate::names::SWAP_MUTATOR => alters!(convert_swap_mutator(&self)),
            crate::names::SCRAMBLE_MUTATOR => alters!(convert_scramble_mutator(&self)),
            crate::names::UNIFORM_MUTATOR => alters!(convert_uniform_mutator(&self)),
            crate::names::INVERSION_MUTATOR => alters!(convert_inversion_mutator(&self)),
            crate::names::CUSTOM_MUTATOR => alters!(convert_custom_mutator(&self)),
            _ => panic!("Invalid alterer type {}", self.component),
        }
    }
}

impl InputTransform<Vec<Box<dyn Alter<BitChromosome>>>> for PyEngineInput {
    fn transform(&self) -> Vec<Box<dyn Alter<BitChromosome>>> {
        if self.input_type != PyEngineInputType::Alterer {
            panic!("Input type {:?} not an alterer", self.input_type);
        }

        match self.component.as_str() {
            crate::names::MULTI_POINT_CROSSOVER => alters!(convert_multi_point_crossover(&self)),
            crate::names::UNIFORM_CROSSOVER => alters!(convert_uniform_crossover(&self)),
            crate::names::SHUFFLE_CROSSOVER => alters!(convert_shuffle_crossover(&self)),
            crate::names::CUSTOM_CROSSOVER => alters!(convert_custom_crossover(&self)),
            crate::names::SWAP_MUTATOR => alters!(convert_swap_mutator(&self)),
            crate::names::SCRAMBLE_MUTATOR => alters!(convert_scramble_mutator(&self)),
            crate::names::UNIFORM_MUTATOR => alters!(convert_uniform_mutator(&self)),
            crate::names::INVERSION_MUTATOR => alters!(convert_inversion_mutator(&self)),
            crate::names::CUSTOM_MUTATOR => alters!(convert_custom_mutator(&self)),
            _ => panic!("Invalid alterer type {}", self.component),
        }
    }
}

impl InputTransform<Vec<Box<dyn Alter<GraphChromosome<Op<f32>>>>>> for PyEngineInput {
    fn transform(&self) -> Vec<Box<dyn Alter<GraphChromosome<Op<f32>>>>> {
        if self.input_type != PyEngineInputType::Alterer {
            panic!("Input type {:?} not an alterer", self.input_type);
        }

        match self.component.as_str() {
            crate::names::GRAPH_CROSSOVER => alters!(convert_graph_crossover(&self)),
            crate::names::GRAPH_MUTATOR => alters!(convert_graph_mutator(&self)),
            crate::names::OPERATION_MUTATOR => alters!(convert_operation_mutator(&self)),
            _ => panic!(
                "Invalid alterer type {} for GraphChromosome",
                self.component
            ),
        }
    }
}

impl InputTransform<Vec<Box<dyn Alter<TreeChromosome<Op<f32>>>>>> for PyEngineInput {
    fn transform(&self) -> Vec<Box<dyn Alter<TreeChromosome<Op<f32>>>>> {
        if self.input_type != PyEngineInputType::Alterer {
            panic!("Input type {:?} not an alterer", self.input_type);
        }

        match self.component.as_str() {
            crate::names::TREE_CROSSOVER => alters!(convert_tree_crossover(&self)),
            crate::names::HOIST_MUTATOR => alters!(convert_hoist_mutator(&self)),
            crate::names::OPERATION_MUTATOR => alters!(convert_operation_mutator(&self)),
            _ => panic!("Invalid alterer type {}", self.component),
        }
    }
}

impl InputTransform<Vec<Box<dyn Alter<PermutationChromosome<usize>>>>> for PyEngineInput {
    fn transform(&self) -> Vec<Box<dyn Alter<PermutationChromosome<usize>>>> {
        if self.input_type != PyEngineInputType::Alterer {
            panic!("Input type {:?} not an alterer", self.input_type);
        }

        match self.component.as_str() {
            crate::names::PARTIALLY_MAPPED_CROSSOVER => {
                alters!(convert_partially_mapped_crossover(&self))
            }
            crate::names::CUSTOM_CROSSOVER => alters!(convert_custom_crossover(&self)),
            crate::names::SWAP_MUTATOR => alters!(convert_swap_mutator(&self)),
            crate::names::SCRAMBLE_MUTATOR => alters!(convert_scramble_mutator(&self)),
            crate::names::UNIFORM_MUTATOR => alters!(convert_uniform_mutator(&self)),
            crate::names::INVERSION_MUTATOR => alters!(convert_inversion_mutator(&self)),
            crate::names::CUSTOM_MUTATOR => alters!(convert_custom_mutator(&self)),
            crate::names::EDGE_RECOMBINE_CROSSOVER => {
                alters!(convert_edge_recombine_crossover(&self))
            }
            _ => panic!("Invalid alterer type {}", self.component),
        }
    }
}

impl InputTransform<Vec<Box<dyn Alter<AnyChromosome<'static>>>>> for PyEngineInput {
    fn transform(&self) -> Vec<Box<dyn Alter<AnyChromosome<'static>>>> {
        if self.input_type != PyEngineInputType::Alterer {
            panic!("Input type {:?} not an alterer", self.input_type);
        }

        match self.component.as_str() {
            crate::names::MULTI_POINT_CROSSOVER => alters!(convert_multi_point_crossover(&self)),
            crate::names::UNIFORM_CROSSOVER => alters!(convert_uniform_crossover(&self)),
            crate::names::SHUFFLE_CROSSOVER => alters!(convert_shuffle_crossover(&self)),
            crate::names::CUSTOM_CROSSOVER => alters!(convert_custom_crossover(&self)),
            crate::names::SWAP_MUTATOR => alters!(convert_swap_mutator(&self)),
            crate::names::SCRAMBLE_MUTATOR => alters!(convert_scramble_mutator(&self)),
            crate::names::UNIFORM_MUTATOR => alters!(convert_uniform_mutator(&self)),
            crate::names::INVERSION_MUTATOR => alters!(convert_inversion_mutator(&self)),
            crate::names::CUSTOM_MUTATOR => alters!(convert_custom_mutator(&self)),
            _ => panic!("Invalid alterer type {}", self.component),
        }
    }
}

fn convert_custom_mutator<C>(input: &PyEngineInput) -> PyMutator<C>
where
    C: Chromosome + Clone,
    PyChromosome: From<C>,
{
    let rate = input.get_f32("rate").unwrap_or(1.0);
    let mutate_func = input
        .extract::<Py<PyAny>>("mutate")
        .expect("Mutate function must be provided");
    let name = input
        .get_string("name")
        .unwrap_or_else(|| "CustomMutator".to_string());
    PyMutator::new(rate, name, mutate_func)
}

fn convert_custom_crossover<C>(input: &PyEngineInput) -> PyCrossover<C>
where
    C: Chromosome + Clone,
    PyChromosome: From<C>,
{
    let rate = input.get_f32("rate").unwrap_or(0.5);
    let crossover_func = input
        .extract::<Py<PyAny>>("crossover")
        .expect("Crossover function must be provided");
    let name = input
        .get_string("name")
        .unwrap_or_else(|| "CustomCrossover".to_string());
    PyCrossover::new(rate, name, crossover_func)
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
    TreeCrossover::new(rate)
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
