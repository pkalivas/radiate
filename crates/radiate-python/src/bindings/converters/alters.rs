use crate::{InputTransform, PyEngineInput, PyEngineInputType};
use radiate::*;

const MULTI_POINT_CROSSOVER: &str = "MultiPointCrossover";
const UNIFORM_CROSSOVER: &str = "UniformCrossover";
const MEAN_CROSSOVER: &str = "MeanCrossover";
const INTERMEDIATE_CROSSOVER: &str = "IntermediateCrossover";
const BLEND_CROSSOVER: &str = "BlendCrossover";
const SHUFFLE_CROSSOVER: &str = "ShuffleCrossover";
const SIMULATED_BINARY_CROSSOVER: &str = "SimulatedBinaryCrossover";
const GRAPH_CROSSOVER: &str = "GraphCrossover";
const PARTIALLY_MAPPED_CROSSOVER: &str = "PartiallyMappedCrossover";

const UNIFORM_MUTATOR: &str = "UniformMutator";
const SCRAMBLE_MUTATOR: &str = "ScrambleMutator";
const SWAP_MUTATOR: &str = "SwapMutator";
const ARITHMETIC_MUTATOR: &str = "ArithmeticMutator";
const GAUSSIAN_MUTATOR: &str = "GaussianMutator";
const GRAPH_MUTATOR: &str = "GraphMutator";
const OPERATION_MUTATOR: &str = "OperationMutator";
const TREE_CROSSOVER: &str = "TreeCrossover";
const HOIST_MUTATOR: &str = "HoistMutator";
const INVERSION_MUTATOR: &str = "InversionMutator";

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
            MULTI_POINT_CROSSOVER => alters!(convert_multi_point_crossover(&self)),
            UNIFORM_CROSSOVER => alters!(convert_uniform_crossover(&self)),
            MEAN_CROSSOVER => alters!(convert_mean_crossover(&self)),
            SHUFFLE_CROSSOVER => alters!(convert_shuffle_crossover(&self)),
            ARITHMETIC_MUTATOR => alters!(convert_arithmetic_mutator(&self)),
            SWAP_MUTATOR => alters!(convert_swap_mutator(&self)),
            SCRAMBLE_MUTATOR => alters!(convert_scramble_mutator(&self)),
            UNIFORM_MUTATOR => alters!(convert_uniform_mutator(&self)),
            INVERSION_MUTATOR => alters!(convert_inversion_mutator(&self)),
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
            MULTI_POINT_CROSSOVER => alters!(convert_multi_point_crossover(&self)),
            UNIFORM_CROSSOVER => alters!(convert_uniform_crossover(&self)),
            MEAN_CROSSOVER => alters!(convert_mean_crossover(&self)),
            INTERMEDIATE_CROSSOVER => alters!(convert_intermediate_crossover(&self)),
            BLEND_CROSSOVER => alters!(convert_blend_crossover(&self)),
            SIMULATED_BINARY_CROSSOVER => alters!(convert_simulated_binary_crossover(&self)),
            GAUSSIAN_MUTATOR => alters!(convert_gaussian_mutator(&self)),
            ARITHMETIC_MUTATOR => alters!(convert_arithmetic_mutator(&self)),
            SWAP_MUTATOR => alters!(convert_swap_mutator(&self)),
            SCRAMBLE_MUTATOR => alters!(convert_scramble_mutator(&self)),
            UNIFORM_MUTATOR => alters!(convert_uniform_mutator(&self)),
            INVERSION_MUTATOR => alters!(convert_inversion_mutator(&self)),
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
            MULTI_POINT_CROSSOVER => alters!(convert_multi_point_crossover(&self)),
            UNIFORM_CROSSOVER => alters!(convert_uniform_crossover(&self)),
            SHUFFLE_CROSSOVER => alters!(convert_shuffle_crossover(&self)),
            SWAP_MUTATOR => alters!(convert_swap_mutator(&self)),
            SCRAMBLE_MUTATOR => alters!(convert_scramble_mutator(&self)),
            UNIFORM_MUTATOR => alters!(convert_uniform_mutator(&self)),
            INVERSION_MUTATOR => alters!(convert_inversion_mutator(&self)),
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
            MULTI_POINT_CROSSOVER => alters!(convert_multi_point_crossover(&self)),
            UNIFORM_CROSSOVER => alters!(convert_uniform_crossover(&self)),
            SHUFFLE_CROSSOVER => alters!(convert_shuffle_crossover(&self)),
            SWAP_MUTATOR => alters!(convert_swap_mutator(&self)),
            SCRAMBLE_MUTATOR => alters!(convert_scramble_mutator(&self)),
            UNIFORM_MUTATOR => alters!(convert_uniform_mutator(&self)),
            INVERSION_MUTATOR => alters!(convert_inversion_mutator(&self)),
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
            GRAPH_CROSSOVER => alters!(convert_graph_crossover(&self)),
            GRAPH_MUTATOR => alters!(convert_graph_mutator(&self)),
            OPERATION_MUTATOR => alters!(convert_operation_mutator(&self)),
            _ => panic!("Invalid alterer type {}", self.component),
        }
    }
}

impl InputTransform<Vec<Box<dyn Alter<TreeChromosome<Op<f32>>>>>> for PyEngineInput {
    fn transform(&self) -> Vec<Box<dyn Alter<TreeChromosome<Op<f32>>>>> {
        if self.input_type != PyEngineInputType::Alterer {
            panic!("Input type {:?} not an alterer", self.input_type);
        }

        match self.component.as_str() {
            TREE_CROSSOVER => alters!(convert_tree_crossover(&self)),
            HOIST_MUTATOR => alters!(convert_hoist_mutator(&self)),
            OPERATION_MUTATOR => alters!(convert_operation_mutator(&self)),
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
            PARTIALLY_MAPPED_CROSSOVER => alters!(convert_partially_mapped_crossover(&self)),
            SWAP_MUTATOR => alters!(convert_swap_mutator(&self)),
            SCRAMBLE_MUTATOR => alters!(convert_scramble_mutator(&self)),
            UNIFORM_MUTATOR => alters!(convert_uniform_mutator(&self)),
            INVERSION_MUTATOR => alters!(convert_inversion_mutator(&self)),
            _ => panic!("Invalid alterer type {}", self.component),
        }
    }
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
