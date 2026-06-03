use crate::PyAnyObject;
use radiate::{
    BitChromosome, CharChromosome, EngineIteratorExt, FloatChromosome, Generation, GeneticEngine,
    GeneticEngineBuilder, Graph, GraphChromosome, IntChromosome, Limit, Op, PermutationChromosome,
    Tree, TreeChromosome,
};
use serde::{Deserialize, Serialize};

type CustomEngine<C> = GeneticEngine<C, PyAnyObject>;
type RegressionEngine<C, T> = GeneticEngine<C, T>;

type CustomBuilder<C, T> = GeneticEngineBuilder<C, T>;
type RegressionBuilder<C, T> = GeneticEngineBuilder<C, T>;

type GenIter<C, T> = Box<dyn Iterator<Item = Generation<C, T>>>;

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

impl EngineHandle {
    pub fn into_step(self, limits: Vec<Limit>) -> StepHandle {
        use EngineHandle::*;
        match self {
            UInt8(eng) => StepHandle::UInt8(eng.iter().limit(limits)),
            UInt16(eng) => StepHandle::UInt16(eng.iter().limit(limits)),
            UInt32(eng) => StepHandle::UInt32(eng.iter().limit(limits)),
            UInt64(eng) => StepHandle::UInt64(eng.iter().limit(limits)),
            Int8(eng) => StepHandle::Int8(eng.iter().limit(limits)),
            Int16(eng) => StepHandle::Int16(eng.iter().limit(limits)),
            Int32(eng) => StepHandle::Int32(eng.iter().limit(limits)),
            Int64(eng) => StepHandle::Int64(eng.iter().limit(limits)),
            Float32(eng) => StepHandle::Float32(eng.iter().limit(limits)),
            Float64(eng) => StepHandle::Float64(eng.iter().limit(limits)),
            Char(eng) => StepHandle::Char(eng.iter().limit(limits)),
            Bit(eng) => StepHandle::Bit(eng.iter().limit(limits)),
            Permutation(eng) => StepHandle::Permutation(eng.iter().limit(limits)),
            Graph(eng) => StepHandle::Graph(eng.iter().limit(limits)),
            Tree(eng) => StepHandle::Tree(eng.iter().limit(limits)),
        }
    }
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

pub enum StepHandle {
    UInt8(GenIter<IntChromosome<u8>, PyAnyObject>),
    UInt16(GenIter<IntChromosome<u16>, PyAnyObject>),
    UInt32(GenIter<IntChromosome<u32>, PyAnyObject>),
    UInt64(GenIter<IntChromosome<u64>, PyAnyObject>),

    Int8(GenIter<IntChromosome<i8>, PyAnyObject>),
    Int16(GenIter<IntChromosome<i16>, PyAnyObject>),
    Int32(GenIter<IntChromosome<i32>, PyAnyObject>),
    Int64(GenIter<IntChromosome<i64>, PyAnyObject>),

    Float32(GenIter<FloatChromosome<f32>, PyAnyObject>),
    Float64(GenIter<FloatChromosome<f64>, PyAnyObject>),

    Char(GenIter<CharChromosome, PyAnyObject>),
    Bit(GenIter<BitChromosome, PyAnyObject>),
    Permutation(GenIter<PermutationChromosome<usize>, PyAnyObject>),

    Graph(GenIter<GraphChromosome<Op<f32>>, Graph<Op<f32>>>),
    Tree(GenIter<TreeChromosome<Op<f32>>, Vec<Tree<Op<f32>>>>),
}
