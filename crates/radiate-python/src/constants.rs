#[macro_export]
macro_rules! define_consts {
    ($($konst:ident = $val:literal;)+) => {
        $(pub(crate) const $konst: &str = $val;)+

        pub fn register(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
            $(m.add(stringify!($konst), $konst)?;)+
            Ok(())
        }
    };
}

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

        SCORE_LIMIT = "ScoreLimit";
        GENERATIONS_LIMIT = "GenerationsLimit";
        SECONDS_LIMIT = "SecondsLimit";
        CONVERGENCE_LIMIT = "ConvergenceLimit";
        METRIC_LIMIT = "MetricLimit";
        EXPR_LIMIT = "ExprLimit";

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
