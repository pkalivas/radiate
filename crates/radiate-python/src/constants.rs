#[macro_export]
macro_rules! define_consts {
    ($($konst:ident = $val:literal;)+) => {
        $(pub(crate) const $konst: &str = $val;)+

        pub(crate) fn register(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
            $(m.add(stringify!($konst), $konst)?;)+
            Ok(())
        }
    };
}

pub use components::*;
pub use loss_functions::*;

pub mod components {
    use pyo3::prelude::*;

    define_consts! {
        TOURNAMENT_SELECTOR = "TournamentSelector";
        ROULETTE_WHEEL_SELECTOR = "RouletteSelector";
        RANK_SELECTOR = "RankSelector";
        STOCHASTIC_UNIVERSAL_SELECTOR = "StochasticUniversalSamplingSelector";
        BOLTZMANN_SELECTOR = "BoltzmannSelector";
        ELITE_SELECTOR = "EliteSelector";
        RANDOM_SELECTOR = "RandomSelector";
        NSGA2_SELECTOR = "NSGA2Selector";
        NSGA3_SELECTOR = "NSGA3Selector";
        TOURNAMENT_NSGA2_SELECTOR = "TournamentNSGA2Selector";
        LINEAR_RANK_SELECTOR = "LinearRankSelector";

        SERIAL_EXECUTOR = "Serial";
        FIXED_SIZED_WORKER_POOL_EXECUTOR = "FixedSizedWorkerPool";
        WORKER_POOL_EXECUTOR = "WorkerPool";

        HAMMING_DISTANCE = "HammingDistance";
        EUCLIDEAN_DISTANCE = "EuclideanDistance";
        COSINE_DISTANCE = "CosineDistance";
        NEAT_DISTANCE = "NeatDistance";

        MULTI_POINT_CROSSOVER = "MultiPointCrossover";
        UNIFORM_CROSSOVER = "UniformCrossover";
        MEAN_CROSSOVER = "MeanCrossover";
        INTERMEDIATE_CROSSOVER = "IntermediateCrossover";
        BLEND_CROSSOVER = "BlendCrossover";
        SHUFFLE_CROSSOVER = "ShuffleCrossover";
        SIMULATED_BINARY_CROSSOVER = "SimulatedBinaryCrossover";
        GRAPH_CROSSOVER = "GraphCrossover";
        PARTIALLY_MAPPED_CROSSOVER = "PartiallyMappedCrossover";
        EDGE_RECOMBINE_CROSSOVER = "EdgeRecombinationCrossover";

        UNIFORM_MUTATOR = "UniformMutator";
        SCRAMBLE_MUTATOR = "ScrambleMutator";
        SWAP_MUTATOR = "SwapMutator";
        ARITHMETIC_MUTATOR = "ArithmeticMutator";
        GAUSSIAN_MUTATOR = "GaussianMutator";
        GRAPH_MUTATOR = "GraphMutator";
        OPERATION_MUTATOR = "OperationMutator";
        TREE_CROSSOVER = "TreeCrossover";
        HOIST_MUTATOR = "HoistMutator";
        INVERSION_MUTATOR = "InversionMutator";
        POLYNOMIAL_MUTATOR = "PolynomialMutator";
        JITTER_MUTATOR = "JitterMutator";

        UNIQUE_SCORE_FILTER = "UniqueScoreFilter";

        ALL_EVENTS = "all";
        START_EVENT = "start_event";
        STOP_EVENT = "stop_event";
        EPOCH_START_EVENT = "epoch_start_event";
        EPOCH_COMPLETE_EVENT = "epoch_complete_event";
        ENGINE_IMPROVEMENT_EVENT = "engine_improvement_event";


    }
}

pub mod loss_functions {
    use pyo3::prelude::*;

    define_consts! {
        MSE_LOSS = "mse";
        MAE_LOSS = "mae";
        CROSS_ENTROPY_LOSS = "xent";
        DIFF_LOSS = "diff";
    }
}

// // The one explicit item in `names` itself — shadows both glob-imported
// // `register`s unambiguously, so this is what lib.rs actually calls.
// pub fn register(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
//     components::register(m)?;
//     loss_functions::register(m)?;
//     Ok(())
// }

// // Selectors
// pub(crate) const TOURNAMENT_SELECTOR: &str = "TournamentSelector";
// pub(crate) const ROULETTE_WHEEL_SELECTOR: &str = "RouletteSelector";
// pub(crate) const RANK_SELECTOR: &str = "RankSelector";
// pub(crate) const STOCHASTIC_UNIVERSAL_SELECTOR: &str = "StochasticUniversalSamplingSelector";
// pub(crate) const BOLTZMANN_SELECTOR: &str = "BoltzmannSelector";
// pub(crate) const ELITE_SELECTOR: &str = "EliteSelector";
// pub(crate) const RANDOM_SELECTOR: &str = "RandomSelector";
// pub(crate) const NSGA2_SELECTOR: &str = "NSGA2Selector";
// pub(crate) const NSGA3_SELECTOR: &str = "NSGA3Selector";
// pub(crate) const TOURNAMENT_NSGA2_SELECTOR: &str = "TournamentNSGA2Selector";
// pub(crate) const LINEAR_RANK_SELECTOR: &str = "LinearRankSelector";

// // Executors
// pub(crate) const SERIAL_EXECUTOR: &str = "Serial";
// pub(crate) const FIXED_SIZED_WORKER_POOL_EXECUTOR: &str = "FixedSizedWorkerPool";
// pub(crate) const WORKER_POOL_EXECUTOR: &str = "WorkerPool";

// // Distances
// pub(crate) const HAMMING_DISTANCE: &str = "HammingDistance";
// pub(crate) const EUCLIDEAN_DISTANCE: &str = "EuclideanDistance";
// pub(crate) const COSINE_DISTANCE: &str = "CosineDistance";
// pub(crate) const NEAT_DISTANCE: &str = "NeatDistance";

// // Crossovers
// pub(crate) const MULTI_POINT_CROSSOVER: &str = "MultiPointCrossover";
// pub(crate) const UNIFORM_CROSSOVER: &str = "UniformCrossover";
// pub(crate) const MEAN_CROSSOVER: &str = "MeanCrossover";
// pub(crate) const INTERMEDIATE_CROSSOVER: &str = "IntermediateCrossover";
// pub(crate) const BLEND_CROSSOVER: &str = "BlendCrossover";
// pub(crate) const SHUFFLE_CROSSOVER: &str = "ShuffleCrossover";
// pub(crate) const SIMULATED_BINARY_CROSSOVER: &str = "SimulatedBinaryCrossover";
// pub(crate) const GRAPH_CROSSOVER: &str = "GraphCrossover";
// pub(crate) const PARTIALLY_MAPPED_CROSSOVER: &str = "PartiallyMappedCrossover";
// pub(crate) const EDGE_RECOMBINE_CROSSOVER: &str = "EdgeRecombinationCrossover";

// // Mutators
// pub(crate) const UNIFORM_MUTATOR: &str = "UniformMutator";
// pub(crate) const SCRAMBLE_MUTATOR: &str = "ScrambleMutator";
// pub(crate) const SWAP_MUTATOR: &str = "SwapMutator";
// pub(crate) const ARITHMETIC_MUTATOR: &str = "ArithmeticMutator";
// pub(crate) const GAUSSIAN_MUTATOR: &str = "GaussianMutator";
// pub(crate) const GRAPH_MUTATOR: &str = "GraphMutator";
// pub(crate) const OPERATION_MUTATOR: &str = "OperationMutator";
// pub(crate) const TREE_CROSSOVER: &str = "TreeCrossover";
// pub(crate) const HOIST_MUTATOR: &str = "HoistMutator";
// pub(crate) const INVERSION_MUTATOR: &str = "InversionMutator";
// pub(crate) const POLYNOMIAL_MUTATOR: &str = "PolynomialMutator";
// pub(crate) const JITTER_MUTATOR: &str = "JitterMutator";

// // Filters
// pub(crate) const UNIQUE_SCORE_FILTER: &str = "UniqueScoreFilter";

// // Events
// pub(crate) const ALL_EVENTS: &str = "all";
// pub(crate) const START_EVENT: &str = "start_event";
// pub(crate) const STOP_EVENT: &str = "stop_event";
// pub(crate) const EPOCH_START_EVENT: &str = "epoch_start_event";
// pub(crate) const EPOCH_COMPLETE_EVENT: &str = "epoch_complete_event";
// pub(crate) const ENGINE_IMPROVEMENT_EVENT: &str = "engine_improvement_event";

// // Loss Functions
// pub(crate) const MSE_LOSS: &str = "mse";
// pub(crate) const MAE_LOSS: &str = "mae";
// pub(crate) const CROSS_ENTROPY_LOSS: &str = "xent";
// pub(crate) const DIFF_LOSS: &str = "diff";
