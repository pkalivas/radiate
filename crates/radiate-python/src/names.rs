// Selectors
pub(crate) const TOURNAMENT_SELECTOR: &str = "TournamentSelector";
pub(crate) const ROULETTE_WHEEL_SELECTOR: &str = "RouletteSelector";
pub(crate) const RANK_SELECTOR: &str = "RankSelector";
pub(crate) const STEADY_STATE_SELECTOR: &str = "SteadyStateSelector";
pub(crate) const STOCHASTIC_UNIVERSAL_SELECTOR: &str = "StochasticUniversalSamplingSelector";
pub(crate) const BOLTZMANN_SELECTOR: &str = "BoltzmannSelector";
pub(crate) const ELITE_SELECTOR: &str = "EliteSelector";
pub(crate) const RANDOM_SELECTOR: &str = "RandomSelector";
pub(crate) const NSGA2_SELECTOR: &str = "NSGA2Selector";
pub(crate) const TOURNAMENT_NSGA2_SELECTOR: &str = "TournamentNSGA2Selector";

// Executors
pub(crate) const SERIAL_EXECUTOR: &str = "Serial";
pub(crate) const FIXED_SIZED_WORKER_POOL_EXECUTOR: &str = "FixedSizedWorkerPool";
pub(crate) const WORKER_POOL_EXECUTOR: &str = "WorkerPool";

// Distances
pub(crate) const HAMMING_DISTANCE: &str = "HammingDistance";
pub(crate) const EUCLIDEAN_DISTANCE: &str = "EuclideanDistance";
pub(crate) const COSINE_DISTANCE: &str = "CosineDistance";
pub(crate) const NEAT_DISTANCE: &str = "NeatDistance";

// Crossovers
pub(crate) const MULTI_POINT_CROSSOVER: &str = "MultiPointCrossover";
pub(crate) const UNIFORM_CROSSOVER: &str = "UniformCrossover";
pub(crate) const MEAN_CROSSOVER: &str = "MeanCrossover";
pub(crate) const INTERMEDIATE_CROSSOVER: &str = "IntermediateCrossover";
pub(crate) const BLEND_CROSSOVER: &str = "BlendCrossover";
pub(crate) const SHUFFLE_CROSSOVER: &str = "ShuffleCrossover";
pub(crate) const SIMULATED_BINARY_CROSSOVER: &str = "SimulatedBinaryCrossover";
pub(crate) const GRAPH_CROSSOVER: &str = "GraphCrossover";
pub(crate) const PARTIALLY_MAPPED_CROSSOVER: &str = "PartiallyMappedCrossover";
pub(crate) const EDGE_RECOMBINE_CROSSOVER: &str = "EdgeRecombinationCrossover";

// Mutators
pub(crate) const UNIFORM_MUTATOR: &str = "UniformMutator";
pub(crate) const SCRAMBLE_MUTATOR: &str = "ScrambleMutator";
pub(crate) const SWAP_MUTATOR: &str = "SwapMutator";
pub(crate) const ARITHMETIC_MUTATOR: &str = "ArithmeticMutator";
pub(crate) const GAUSSIAN_MUTATOR: &str = "GaussianMutator";
pub(crate) const GRAPH_MUTATOR: &str = "GraphMutator";
pub(crate) const OPERATION_MUTATOR: &str = "OperationMutator";
pub(crate) const TREE_CROSSOVER: &str = "TreeCrossover";
pub(crate) const HOIST_MUTATOR: &str = "HoistMutator";
pub(crate) const INVERSION_MUTATOR: &str = "InversionMutator";
pub(crate) const POLYNOMIAL_MUTATOR: &str = "PolynomialMutator";
pub(crate) const JITTER_MUTATOR: &str = "JitterMutator";

// Events
pub const START_EVENT: &str = "start_event";
pub const STOP_EVENT: &str = "stop_event";
pub const EPOCH_START_EVENT: &str = "epoch_start_event";
pub const EPOCH_COMPLETE_EVENT: &str = "epoch_complete_event";
pub const ENGINE_IMPROVEMENT_EVENT: &str = "engine_improvement_event";
