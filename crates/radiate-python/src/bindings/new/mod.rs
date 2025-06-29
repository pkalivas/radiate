mod builder;
mod converters;
mod engine;
mod inputs;

pub use builder::*;
pub use converters::*;
pub use engine::*;
pub use inputs::*;

use crate::ObjectValue;
use radiate::{
    BitChromosome, CharChromosome, FloatChromosome, Generation, GeneticEngine,
    GeneticEngineBuilder, Graph, GraphChromosome, IntChromosome, Op, ParetoGeneration,
};

type SingleObjBuilder<C, T> = GeneticEngineBuilder<C, T, Generation<C, T>>;
type MultiObjBuilder<C, T> = GeneticEngineBuilder<C, T, ParetoGeneration<C>>;
type RegressionBuilder<C, T> = GeneticEngineBuilder<C, T, Generation<C, T>>;

type SingleObjectiveEngine<C> = GeneticEngine<C, ObjectValue, Generation<C, ObjectValue>>;
type MultiObjectiveEngine<C> = GeneticEngine<C, ObjectValue, ParetoGeneration<C>>;
type RegressionEngine<C, T> = GeneticEngine<C, T, Generation<C, T>>;

pub enum EngineBuilderHandle {
    Empty,
    Int(SingleObjBuilder<IntChromosome<i32>, ObjectValue>),
    Float(SingleObjBuilder<FloatChromosome, ObjectValue>),
    Char(SingleObjBuilder<CharChromosome, ObjectValue>),
    Bit(SingleObjBuilder<BitChromosome, ObjectValue>),
    IntMulti(MultiObjBuilder<IntChromosome<i32>, ObjectValue>),
    FloatMulti(MultiObjBuilder<FloatChromosome, ObjectValue>),
    GraphRegression(RegressionBuilder<GraphChromosome<Op<f32>>, Graph<Op<f32>>>),
}

pub enum EngineHandle {
    Int(SingleObjectiveEngine<IntChromosome<i32>>),
    Float(SingleObjectiveEngine<FloatChromosome>),
    Char(SingleObjectiveEngine<CharChromosome>),
    Bit(SingleObjectiveEngine<BitChromosome>),
    IntMulti(MultiObjectiveEngine<IntChromosome<i32>>),
    FloatMulti(MultiObjectiveEngine<FloatChromosome>),
    GraphRegression(RegressionEngine<GraphChromosome<Op<f32>>, Graph<Op<f32>>>),
}

pub enum EpochHandle {
    Int(Generation<IntChromosome<i32>, ObjectValue>),
    Float(Generation<FloatChromosome, ObjectValue>),
    Char(Generation<CharChromosome, ObjectValue>),
    Bit(Generation<BitChromosome, ObjectValue>),
    IntMulti(ParetoGeneration<IntChromosome<i32>>),
    FloatMulti(ParetoGeneration<FloatChromosome>),
    GraphRegression(Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>),
}
