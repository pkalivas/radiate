pub mod alter;
pub mod codecs;
pub mod distance;
pub mod domain;
pub mod engine;
pub mod evaluator;
pub mod fitness;
pub mod genome;
pub mod objectives;
pub mod problem;
pub mod replacement;
pub mod selector;
pub mod stats;

pub use alter::{Alter, AlterAction, AlterResult, Crossover, Mutate};
pub use codecs::{
    BitCodec, CharCodec, Codec, FloatCodec, FnCodec, IntCodec, PermutationCodec, SubSetCodec,
};
pub use distance::{CosineDistance, Diversity, EuclideanDistance, HammingDistance};
pub use domain::*;
pub use engine::{Engine, EngineExt};
pub use evaluator::{Evaluator, FitnessEvaluator};
pub use executor::Executor;
pub use fitness::{CompositeFitnessFn, FitnessFunction, NoveltySearch};
pub use genome::*;
pub use objectives::{Front, Objective, Optimize, Score, pareto};
pub use problem::{EngineProblem, Problem};
pub use replacement::{EncodeReplace, PopulationSampleReplace, ReplacementStrategy};
pub use selector::Select;
pub use stats::*;

pub mod prelude {
    pub use super::alter::{Alter, Crossover, Mutate};
    pub use super::codecs::{
        BitCodec, CharCodec, Codec, FloatCodec, FnCodec, IntCodec, PermutationCodec, SubSetCodec,
    };
    pub use super::distance::{CosineDistance, Diversity, EuclideanDistance, HammingDistance};
    pub use super::domain::random_provider;
    pub use super::engine::{Engine, EngineExt};
    pub use super::executor::Executor;
    pub use super::fitness::{CompositeFitnessFn, FitnessFunction, NoveltySearch};
    pub use super::genome::{
        ArithmeticGene, BitChromosome, BitGene, CharChromosome, CharGene, Chromosome,
        FloatChromosome, FloatGene, Gene, IntChromosome, IntGene, Integer, Valid,
    };
    pub use super::objectives::{Front, Objective, Optimize, Score, pareto};
    pub use super::problem::{EngineProblem, Problem};
    pub use super::replacement::{EncodeReplace, PopulationSampleReplace, ReplacementStrategy};
    pub use super::selector::Select;
    pub use super::stats::{Distribution, Metric, MetricSet, Statistic, TimeStatistic};
}
