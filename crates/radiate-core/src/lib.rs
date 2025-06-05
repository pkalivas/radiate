pub mod alter;
pub mod audit;
pub mod codecs;
pub mod diversity;
pub mod domain;
pub mod engine;
mod executor;
pub mod genome;
pub mod objectives;
pub mod problem;
pub mod replacement;
pub mod selector;
pub mod stats;

pub use alter::{Alter, AlterAction, AlterResult, Crossover, Mutate};
pub use audit::{Audit, MetricAudit};
pub use codecs::{
    BitCodec, CharCodec, Codec, FloatCodec, FnCodec, IntCodec, PermutationCodec, SubSetCodec,
};
pub use diversity::{Diversity, EuclideanDistance, HammingDistance};
pub use domain::*;
pub use engine::{Engine, EngineExt, EngineStep, Epoch};
pub use executor::Executor;
pub use genome::*;
pub use objectives::{Front, Objective, Optimize, ParetoFront, Score, pareto};
pub use problem::{EngineProblem, Problem};
pub use replacement::{EncodeReplace, PopulationSampleReplace, ReplacementStrategy};
pub use selector::{ProbabilityWheelIterator, Select};
pub use stats::*;

pub mod prelude {
    pub use super::alter::{Alter, Crossover, Mutate};
    pub use super::audit::{Audit, MetricAudit};
    pub use super::codecs::{
        BitCodec, CharCodec, Codec, FloatCodec, FnCodec, IntCodec, PermutationCodec, SubSetCodec,
    };
    pub use super::diversity::{Diversity, EuclideanDistance, HammingDistance};
    pub use super::domain::random_provider;
    pub use super::engine::{Engine, EngineExt, EngineStep, Epoch};
    pub use super::executor::Executor;
    pub use super::genome::{
        ArithmeticGene, BitChromosome, BitGene, CharChromosome, CharGene, Chromosome,
        FloatChromosome, FloatGene, Gene, IntChromosome, IntGene, Integer, Valid,
    };
    pub use super::objectives::{Front, Objective, Optimize, ParetoFront, Score, pareto};
    pub use super::problem::{EngineProblem, Problem};
    pub use super::replacement::{EncodeReplace, PopulationSampleReplace, ReplacementStrategy};
    pub use super::selector::{ProbabilityWheelIterator, Select};
    pub use super::stats::{Distribution, Metric, MetricSet, Statistic, TimeStatistic};
}
