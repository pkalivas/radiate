mod builder;
mod codec;
mod converters;
mod engine;
mod epoch;
mod fitness;
mod functions;
mod gene;
mod gp;
mod inputs;
mod metric;
mod subscriber;

pub use builder::*;
pub use codec::{
    PyAnyCodec, PyBitCodec, PyCharCodec, PyCodec, PyFloatCodec, PyGraphCodec, PyIntCodec,
    PyPermutationCodec, PyTreeCodec,
};
pub use converters::InputTransform;
pub use engine::PyEngine;
pub use epoch::PyGeneration;
pub use fitness::{PyFitnessFn, PyFitnessInner, PyNoveltySearch};
pub use functions::*;
pub use gene::{PyChromosome, PyGene, PyGeneType, PyGenotype, PyPhenotype, PyPopulation};
pub use gp::{PyGraph, PyGraphNode, PyTree, PyTreeNode};
pub use inputs::{PyEngineInput, PyEngineInputType};

pub use metric::PyMetricSet;
pub use subscriber::PySubscriber;

use crate::ObjectValue;
use radiate::{
    BitChromosome, CharChromosome, FloatChromosome, Generation, GeneticEngine,
    GeneticEngineBuilder, Graph, GraphChromosome, IntChromosome, Op, PermutationChromosome, Tree,
    TreeChromosome,
};

type SingleObjBuilder<C, T> = GeneticEngineBuilder<C, T>;
type RegressionBuilder<C, T> = GeneticEngineBuilder<C, T>;

type SingleObjectiveEngine<C> = GeneticEngine<C, ObjectValue>;
type RegressionEngine<C, T> = GeneticEngine<C, T>;

pub enum EngineBuilderHandle {
    Empty,
    Int(SingleObjBuilder<IntChromosome<i32>, ObjectValue>),
    Float(SingleObjBuilder<FloatChromosome, ObjectValue>),
    Char(SingleObjBuilder<CharChromosome, ObjectValue>),
    Bit(SingleObjBuilder<BitChromosome, ObjectValue>),
    Permutation(SingleObjBuilder<PermutationChromosome<usize>, ObjectValue>),
    Graph(RegressionBuilder<GraphChromosome<Op<f32>>, Graph<Op<f32>>>),
    Tree(RegressionBuilder<TreeChromosome<Op<f32>>, Vec<Tree<Op<f32>>>>),
}

pub enum EngineHandle {
    Int(SingleObjectiveEngine<IntChromosome<i32>>),
    Float(SingleObjectiveEngine<FloatChromosome>),
    Char(SingleObjectiveEngine<CharChromosome>),
    Bit(SingleObjectiveEngine<BitChromosome>),
    Permutation(SingleObjectiveEngine<PermutationChromosome<usize>>),
    Graph(RegressionEngine<GraphChromosome<Op<f32>>, Graph<Op<f32>>>),
    Tree(RegressionEngine<TreeChromosome<Op<f32>>, Vec<Tree<Op<f32>>>>),
}

pub enum EpochHandle {
    Int(Generation<IntChromosome<i32>, ObjectValue>),
    Float(Generation<FloatChromosome, ObjectValue>),
    Char(Generation<CharChromosome, ObjectValue>),
    Bit(Generation<BitChromosome, ObjectValue>),
    Permutation(Generation<PermutationChromosome<usize>, ObjectValue>),
    Graph(Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>),
    Tree(Generation<TreeChromosome<Op<f32>>, Vec<Tree<Op<f32>>>>),
}
