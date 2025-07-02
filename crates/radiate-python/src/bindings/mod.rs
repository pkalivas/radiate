mod builder;
mod codec;
mod converters;
mod engine;
mod epoch;
mod gene;
mod inputs;
mod metric;
mod ops;
mod problem;
mod subscriber;

pub use builder::PyEngineBuilder;
pub use codec::{
    PyAnyCodec, PyBitCodec, PyCharCodec, PyCodec, PyFloatCodec, PyGraph, PyGraphCodec, PyIntCodec,
    PyTree, PyTreeCodec,
};
pub use converters::InputConverter;
pub use engine::PyEngine;
pub use epoch::PyGeneration;
pub use gene::{PyChromosome, PyGene, PyGeneType, PyGenotype, PyPhenotype, PyPopulation};
pub use inputs::{PyEngineInput, PyEngineInputType};

pub use metric::PyMetricSet;
pub use problem::PyProblemBuilder;
pub use subscriber::PySubscriber;

use crate::ObjectValue;
use radiate::{
    BitChromosome, CharChromosome, FloatChromosome, Generation, GeneticEngine,
    GeneticEngineBuilder, Graph, GraphChromosome, IntChromosome, Op, ParetoGeneration, Tree,
    TreeChromosome,
};

type SingleObjBuilder<C, T> = GeneticEngineBuilder<C, T, Generation<C, T>>;
type MultiObjBuilder<C, T> = GeneticEngineBuilder<C, T, ParetoGeneration<C>>;
type RegressionBuilder<C, T> = GeneticEngineBuilder<C, T, Generation<C, T>>;

type SingleObjectiveEngine<C> = GeneticEngine<C, ObjectValue>;
type MultiObjectiveEngine<C> = GeneticEngine<C, ObjectValue>;
type RegressionEngine<C, T> = GeneticEngine<C, T>;

pub enum EngineBuilderHandle {
    Empty,
    Int(SingleObjBuilder<IntChromosome<i32>, ObjectValue>),
    Float(SingleObjBuilder<FloatChromosome, ObjectValue>),
    Char(SingleObjBuilder<CharChromosome, ObjectValue>),
    Bit(SingleObjBuilder<BitChromosome, ObjectValue>),
    IntMulti(MultiObjBuilder<IntChromosome<i32>, ObjectValue>),
    FloatMulti(MultiObjBuilder<FloatChromosome, ObjectValue>),
    GraphRegression(RegressionBuilder<GraphChromosome<Op<f32>>, Graph<Op<f32>>>),
    TreeRegression(RegressionBuilder<TreeChromosome<Op<f32>>, Vec<Tree<Op<f32>>>>),
}

pub enum EngineHandle {
    Int(SingleObjectiveEngine<IntChromosome<i32>>),
    Float(SingleObjectiveEngine<FloatChromosome>),
    Char(SingleObjectiveEngine<CharChromosome>),
    Bit(SingleObjectiveEngine<BitChromosome>),
    IntMulti(MultiObjectiveEngine<IntChromosome<i32>>),
    FloatMulti(MultiObjectiveEngine<FloatChromosome>),
    GraphRegression(RegressionEngine<GraphChromosome<Op<f32>>, Graph<Op<f32>>>),
    TreeRegression(RegressionEngine<TreeChromosome<Op<f32>>, Vec<Tree<Op<f32>>>>),
}

pub enum EpochHandle {
    Int(Generation<IntChromosome<i32>, ObjectValue>),
    Float(Generation<FloatChromosome, ObjectValue>),
    Char(Generation<CharChromosome, ObjectValue>),
    Bit(Generation<BitChromosome, ObjectValue>),
    IntMulti(ParetoGeneration<IntChromosome<i32>>),
    FloatMulti(ParetoGeneration<FloatChromosome>),
    GraphRegression(Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>),
    TreeRegression(Generation<TreeChromosome<Op<f32>>, Vec<Tree<Op<f32>>>>),
}
