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
    PyBitCodec, PyCharCodec, PyCodec, PyFloatCodec, PyGraphCodec, PyIntCodec, PyPermutationCodec,
    PyTreeCodec,
};
pub use converters::InputTransform;
pub use engine::PyEngine;
pub use epoch::PyGeneration;
pub use fitness::{PyFitnessFn, PyFitnessInner, PyNoveltySearch};
pub use functions::*;
pub use gene::{
    PyChromosome, PyGene, PyGeneType, PyGenotype, PyPhenotype, PyPopulation, PySpecies,
};
pub use gp::{PyGraph, PyTree};
pub use inputs::{PyEngineInput, PyEngineInputType};

pub use metric::PyMetricSet;
pub use subscriber::PySubscriber;

use crate::PyAnyObject;
use radiate::{
    BitChromosome, CharChromosome, FloatChromosome, Generation, GeneticEngine,
    GeneticEngineBuilder, Graph, GraphChromosome, IntChromosome, Op, PermutationChromosome, Tree,
    TreeChromosome,
};

type SingleObjBuilder<C, T> = GeneticEngineBuilder<C, T>;
type RegressionBuilder<C, T> = GeneticEngineBuilder<C, T>;

type SingleObjectiveEngine<C> = GeneticEngine<C, PyAnyObject>;
type RegressionEngine<C, T> = GeneticEngine<C, T>;

pub enum EngineBuilderHandle {
    Empty,
    Int(SingleObjBuilder<IntChromosome<i32>, PyAnyObject>),
    Float(SingleObjBuilder<FloatChromosome, PyAnyObject>),
    Char(SingleObjBuilder<CharChromosome, PyAnyObject>),
    Bit(SingleObjBuilder<BitChromosome, PyAnyObject>),
    Permutation(SingleObjBuilder<PermutationChromosome<usize>, PyAnyObject>),
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
    Int(Generation<IntChromosome<i32>, PyAnyObject>),
    Float(Generation<FloatChromosome, PyAnyObject>),
    Char(Generation<CharChromosome, PyAnyObject>),
    Bit(Generation<BitChromosome, PyAnyObject>),
    Permutation(Generation<PermutationChromosome<usize>, PyAnyObject>),
    Graph(Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>),
    Tree(Generation<TreeChromosome<Op<f32>>, Vec<Tree<Op<f32>>>>),
}
