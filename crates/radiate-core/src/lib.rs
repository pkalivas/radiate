pub mod alter;
pub mod codecs;
pub mod diversity;
pub mod domain;
pub mod engine;
pub mod error;
pub mod evaluator;
pub mod fitness;
pub mod genome;
pub mod objectives;
pub mod problem;
pub mod rate;
pub mod replacement;
pub mod selector;
pub mod stats;

use radiate_error::Result;
pub use radiate_error::{RadiateError, ensure, radiate_err};

pub use alter::{AlterResult, Alterer, Crossover, Mutate};
pub use codecs::{
    BitCodec, CharCodec, Codec, FloatCodec, FnCodec, IntCodec, PermutationCodec, SubSetCodec,
};
pub use diversity::{CosineDistance, Diversity, EuclideanDistance, HammingDistance};
pub use domain::*;
pub use engine::{Engine, EngineExt};
pub use evaluator::{BatchFitnessEvaluator, Evaluator, FitnessEvaluator};
pub use executor::Executor;
pub use fitness::{BatchFitnessFunction, CompositeFitnessFn, FitnessFunction, NoveltySearch};
pub use genome::*;
pub use objectives::{Front, Objective, Optimize, Score, pareto};
pub use problem::{BatchEngineProblem, EngineProblem, Problem};
pub use rate::Rate;
pub use replacement::{EncodeReplace, PopulationSampleReplace, ReplacementStrategy};
pub use selector::Select;
pub use stats::{
    Distribution, Metric, MetricSet, MetricUpdate, Statistic, TimeStatistic, metric_names,
    render_dashboard, render_full,
};

pub mod prelude {
    pub use radiate_error::*;

    pub use super::alter::{Alterer, Crossover, Mutate};
    pub use super::codecs::{
        BitCodec, CharCodec, Codec, FloatCodec, FnCodec, IntCodec, PermutationCodec, SubSetCodec,
    };
    pub use super::diversity::{CosineDistance, Diversity, EuclideanDistance, HammingDistance};
    pub use super::domain::random_provider;
    pub use super::engine::{Engine, EngineExt};
    pub use super::evaluator::{BatchFitnessEvaluator, Evaluator, FitnessEvaluator};
    pub use super::executor::Executor;
    pub use super::fitness::{
        BatchFitnessFunction, CompositeFitnessFn, FitnessFunction, NoveltySearch,
    };
    pub use super::genome::{
        ArithmeticGene, BitChromosome, BitGene, BoundedGene, CharChromosome, CharGene, Chromosome,
        FloatChromosome, FloatGene, Gene, IntChromosome, IntGene, Integer, Valid,
    };
    pub use super::objectives::{Front, Objective, Optimize, Score, pareto};
    pub use super::problem::{BatchEngineProblem, EngineProblem, Problem};
    pub use super::replacement::{EncodeReplace, PopulationSampleReplace, ReplacementStrategy};
    pub use super::selector::Select;
    pub use super::stats::{Distribution, Metric, MetricSet, Statistic, TimeStatistic};
}
