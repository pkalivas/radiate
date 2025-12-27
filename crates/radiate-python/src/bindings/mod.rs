mod builder;
mod codec;
mod converters;
mod engine;
mod epoch;
mod fitness;
mod functions;
mod genome;
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
pub use engine::{PyEngine, PyEngineRunOption};
pub use epoch::PyGeneration;
pub use fitness::{PyFitnessFn, PyFitnessInner, PyNoveltySearch};
pub use functions::*;
pub use genome::*;
pub use gp::{PyGraph, PyTree};
pub use inputs::{PyEngineInput, PyEngineInputType};
pub use metric::{PyMetric, PyMetricSet, PyTagKind};
pub use subscriber::{PyEngineEvent, PySubscriber};

use crate::{AnyChromosome, PyAnyObject};
use radiate::{
    BitChromosome, CharChromosome, FloatChromosome, Generation, GeneticEngine,
    GeneticEngineBuilder, Graph, GraphChromosome, IntChromosome, Op, PermutationChromosome, Tree,
    TreeChromosome,
};
use serde::{Deserialize, Serialize};

type CustomBuilder<C, T> = GeneticEngineBuilder<C, T>;
type RegressionBuilder<C, T> = GeneticEngineBuilder<C, T>;

type CustomEngine<C> = GeneticEngine<C, PyAnyObject>;
type RegressionEngine<C, T> = GeneticEngine<C, T>;

pub enum EngineBuilderHandle {
    Empty,
    Int(CustomBuilder<IntChromosome<i32>, PyAnyObject>),
    Float(CustomBuilder<FloatChromosome, PyAnyObject>),
    Char(CustomBuilder<CharChromosome, PyAnyObject>),
    Bit(CustomBuilder<BitChromosome, PyAnyObject>),
    Permutation(CustomBuilder<PermutationChromosome<usize>, PyAnyObject>),
    Any(CustomBuilder<AnyChromosome<'static>, PyAnyObject>),
    Graph(RegressionBuilder<GraphChromosome<Op<f32>>, Graph<Op<f32>>>),
    Tree(RegressionBuilder<TreeChromosome<Op<f32>>, Vec<Tree<Op<f32>>>>),
}

pub enum EngineHandle {
    Int(CustomEngine<IntChromosome<i32>>),
    Float(CustomEngine<FloatChromosome>),
    Char(CustomEngine<CharChromosome>),
    Bit(CustomEngine<BitChromosome>),
    Any(CustomEngine<AnyChromosome<'static>>),
    Permutation(CustomEngine<PermutationChromosome<usize>>),
    Graph(RegressionEngine<GraphChromosome<Op<f32>>, Graph<Op<f32>>>),
    Tree(RegressionEngine<TreeChromosome<Op<f32>>, Vec<Tree<Op<f32>>>>),
}

#[derive(Clone, Serialize, Deserialize)]
pub enum EpochHandle {
    Int(Generation<IntChromosome<i32>, PyAnyObject>),
    Float(Generation<FloatChromosome, PyAnyObject>),
    Char(Generation<CharChromosome, PyAnyObject>),
    Bit(Generation<BitChromosome, PyAnyObject>),
    Any(Generation<AnyChromosome<'static>, PyAnyObject>),
    Permutation(Generation<PermutationChromosome<usize>, PyAnyObject>),
    Graph(Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>),
    Tree(Generation<TreeChromosome<Op<f32>>, Vec<Tree<Op<f32>>>>),
}
