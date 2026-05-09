// Make `radiate_core` resolvable from inside this crate so that derive macros
// (which emit `::radiate_core::...` paths) work for types defined in this
// crate too.
extern crate self as radiate_core;

pub mod alter;
pub mod codecs;
pub mod diversity;
pub mod domain;
pub mod engine;
pub mod error;
pub mod evaluator;
pub mod fitness;
pub mod freeze;
pub mod genome;
pub mod lineage;
pub mod objectives;
pub mod problem;
pub mod rate;
pub mod replacement;
pub mod selector;
pub mod stats;

// pub use radiate_derive::Freeze;
// // `freeze` is also a module name above (in the type namespace); the proc-macro
// // attribute lives in the macro namespace so they coexist without conflict.
// pub use radiate_derive::freeze;

use radiate_error::Result;
pub use radiate_error::{RadiateError, ensure, radiate_err};

pub use alter::{AlterContext, AlterResult, Alterer, Crossover, Mutate};
pub use codecs::{
    BitCodec, CharCodec, Codec, FloatCodec, FnCodec, IntCodec, PermutationCodec, SubSetCodec,
};
pub use diversity::{CosineDistance, Diversity, EuclideanDistance, HammingDistance};
pub use domain::*;
pub use engine::{Engine, EngineExt};
pub use evaluator::{BatchFitnessEvaluator, Evaluator, FitnessEvaluator};
pub use executor::Executor;
pub use fitness::{
    BatchFitnessFunction, BatchedFn, CompositeFitnessFn, FitnessFunction, NoveltySearch,
};
pub use freeze::{Freezable, Frozen, FrozenMap};
pub use genome::*;
pub use lineage::{Lineage, LineageEvent, LineageUpdate};
pub use objectives::{Front, Objective, Optimize, Score, pareto};
pub use problem::{BatchEngineProblem, EngineProblem, Problem};
pub use radiate_utils::{AnyValue, DataType, Field, dtype, dtype_names, value};
pub use rate::Rate;
pub use replacement::{EncodeReplace, PopulationSampleReplace, ReplacementStrategy};
pub use selector::Select;
pub use stats::{
    Evaluate, Expr, ExprProjection, Metric, MetricSet, MetricUpdate, NamedExpr, SelectExpr,
    expression::expr, metric_names, render_dashboard, render_full,
};

pub mod prelude {
    pub use radiate_error::*;

    pub use super::alter::{AlterContext, Alterer, Crossover, Mutate};
    pub use super::codecs::{
        BitCodec, CharCodec, Codec, FloatCodec, FnCodec, IntCodec, PermutationCodec, SubSetCodec,
    };
    pub use super::diversity::{CosineDistance, Diversity, EuclideanDistance, HammingDistance};
    pub use super::domain::random_provider;
    pub use super::engine::{Engine, EngineExt};
    pub use super::evaluator::{BatchFitnessEvaluator, Evaluator, FitnessEvaluator};
    pub use super::executor::Executor;
    pub use super::fitness::{
        BatchFitnessFunction, BatchedFn, CompositeFitnessFn, FitnessFunction, NoveltySearch,
    };
    pub use super::genome::{
        ArithmeticGene, BitChromosome, BitGene, BoundedGene, CharChromosome, CharGene, Chromosome,
        FloatChromosome, FloatGene, Gene, IntChromosome, IntGene, Valid,
    };
    pub use super::objectives::{Front, Objective, Optimize, Score, pareto};
    pub use super::problem::{BatchEngineProblem, EngineProblem, Problem};
    pub use super::replacement::{EncodeReplace, PopulationSampleReplace, ReplacementStrategy};
    pub use super::selector::Select;
    pub use super::stats::{Metric, MetricSet};
}
