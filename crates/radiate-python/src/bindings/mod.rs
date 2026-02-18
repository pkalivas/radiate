mod builder;
mod codec;
mod converters;
mod datatype;
mod engine;
mod epoch;
mod fitness;
mod front;
mod functions;
mod genome;
mod gp;
mod inputs;
mod metric;
mod rate;
mod subscriber;

pub use builder::*;
pub use codec::{
    PyBitCodec, PyCharCodec, PyCodec, PyFloatCodec, PyGraphCodec, PyIntCodec, PyPermutationCodec,
    PyTreeCodec,
};
pub use converters::InputTransform;
pub use datatype::{
    _get_dtype_max, _get_dtype_min, DataType, Field, dtype, dtype_from_str, dtype_names,
};
pub use engine::{PyEngine, PyEngineRunOption};
pub use epoch::PyGeneration;
pub use fitness::{PyFitnessFn, PyFitnessInner, PyNoveltySearch};
pub use front::{PyFront, PyFrontValue};
pub use functions::*;
pub use genome::*;
pub use gp::{
    _activation_ops, _all_ops, _create_op, _edge_ops, PyAccuracy, PyGraph, PyOp, PyTree,
    py_accuracy,
};
pub use inputs::{PyEngineInput, PyEngineInputType};
pub use metric::{PyMetric, PyMetricSet};
use pyo3::{Py, Python, sync::PyOnceLock, types::PyModule};
pub use rate::PyRate;
pub use subscriber::{PyEngineEvent, PySubscriber};

use crate::PyAnyObject;
use radiate::{
    BitChromosome, CharChromosome, FloatChromosome, Generation, GeneticEngine,
    GeneticEngineBuilder, Graph, GraphChromosome, IntChromosome, Op, PermutationChromosome, Tree,
    TreeChromosome,
};
use serde::{Deserialize, Serialize};

static RADIATE: PyOnceLock<Py<PyModule>> = PyOnceLock::new();

pub fn radiate(py: Python<'_>) -> &Py<PyModule> {
    RADIATE.get_or_init(py, || py.import("radiate").unwrap().unbind())
}

type CustomBuilder<C, T> = GeneticEngineBuilder<C, T>;
type RegressionBuilder<C, T> = GeneticEngineBuilder<C, T>;

type CustomEngine<C> = GeneticEngine<C, PyAnyObject>;
type RegressionEngine<C, T> = GeneticEngine<C, T>;

pub enum EngineBuilderHandle {
    Empty,

    UInt8(CustomBuilder<IntChromosome<u8>, PyAnyObject>),
    UInt16(CustomBuilder<IntChromosome<u16>, PyAnyObject>),
    UInt32(CustomBuilder<IntChromosome<u32>, PyAnyObject>),
    UInt64(CustomBuilder<IntChromosome<u64>, PyAnyObject>),

    Int8(CustomBuilder<IntChromosome<i8>, PyAnyObject>),
    Int16(CustomBuilder<IntChromosome<i16>, PyAnyObject>),
    Int32(CustomBuilder<IntChromosome<i32>, PyAnyObject>),
    Int64(CustomBuilder<IntChromosome<i64>, PyAnyObject>),

    Float32(CustomBuilder<FloatChromosome<f32>, PyAnyObject>),
    Float64(CustomBuilder<FloatChromosome<f64>, PyAnyObject>),

    Char(CustomBuilder<CharChromosome, PyAnyObject>),
    Bit(CustomBuilder<BitChromosome, PyAnyObject>),
    Permutation(CustomBuilder<PermutationChromosome<usize>, PyAnyObject>),
    Graph(RegressionBuilder<GraphChromosome<Op<f32>>, Graph<Op<f32>>>),
    Tree(RegressionBuilder<TreeChromosome<Op<f32>>, Vec<Tree<Op<f32>>>>),
}

pub enum EngineHandle {
    UInt8(CustomEngine<IntChromosome<u8>>),
    UInt16(CustomEngine<IntChromosome<u16>>),
    UInt32(CustomEngine<IntChromosome<u32>>),
    UInt64(CustomEngine<IntChromosome<u64>>),

    Int8(CustomEngine<IntChromosome<i8>>),
    Int16(CustomEngine<IntChromosome<i16>>),
    Int32(CustomEngine<IntChromosome<i32>>),
    Int64(CustomEngine<IntChromosome<i64>>),

    Float32(CustomEngine<FloatChromosome<f32>>),
    Float64(CustomEngine<FloatChromosome<f64>>),

    Char(CustomEngine<CharChromosome>),
    Bit(CustomEngine<BitChromosome>),
    Permutation(CustomEngine<PermutationChromosome<usize>>),
    Graph(RegressionEngine<GraphChromosome<Op<f32>>, Graph<Op<f32>>>),
    Tree(RegressionEngine<TreeChromosome<Op<f32>>, Vec<Tree<Op<f32>>>>),
}

#[derive(Clone, Serialize, Deserialize)]
pub enum EpochHandle {
    UInt8(Generation<IntChromosome<u8>, PyAnyObject>),
    UInt16(Generation<IntChromosome<u16>, PyAnyObject>),
    UInt32(Generation<IntChromosome<u32>, PyAnyObject>),
    UInt64(Generation<IntChromosome<u64>, PyAnyObject>),

    Int8(Generation<IntChromosome<i8>, PyAnyObject>),
    Int16(Generation<IntChromosome<i16>, PyAnyObject>),
    Int32(Generation<IntChromosome<i32>, PyAnyObject>),
    Int64(Generation<IntChromosome<i64>, PyAnyObject>),

    Float32(Generation<FloatChromosome<f32>, PyAnyObject>),
    Float64(Generation<FloatChromosome<f64>, PyAnyObject>),

    Char(Generation<CharChromosome, PyAnyObject>),
    Bit(Generation<BitChromosome, PyAnyObject>),
    Permutation(Generation<PermutationChromosome<usize>, PyAnyObject>),
    Graph(Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>),
    Tree(Generation<TreeChromosome<Op<f32>>, Vec<Tree<Op<f32>>>>),
}
