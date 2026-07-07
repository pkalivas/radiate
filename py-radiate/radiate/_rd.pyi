from collections.abc import Callable
from enum import StrEnum
from typing import Final

class _Components:
    TOURNAMENT_SELECTOR: Final[str]
    ROULETTE_WHEEL_SELECTOR: Final[str]
    RANK_SELECTOR: Final[str]
    STOCHASTIC_UNIVERSAL_SELECTOR: Final[str]
    BOLTZMANN_SELECTOR: Final[str]
    ELITE_SELECTOR: Final[str]
    RANDOM_SELECTOR: Final[str]
    NSGA2_SELECTOR: Final[str]
    NSGA3_SELECTOR: Final[str]
    TOURNAMENT_NSGA2_SELECTOR: Final[str]
    LINEAR_RANK_SELECTOR: Final[str]

    SERIAL_EXECUTOR: Final[str]
    FIXED_SIZED_WORKER_POOL_EXECUTOR: Final[str]
    WORKER_POOL_EXECUTOR: Final[str]

    HAMMING_DISTANCE: Final[str]
    EUCLIDEAN_DISTANCE: Final[str]
    COSINE_DISTANCE: Final[str]
    NEAT_DISTANCE: Final[str]

    MULTI_POINT_CROSSOVER: Final[str]
    UNIFORM_CROSSOVER: Final[str]
    MEAN_CROSSOVER: Final[str]
    INTERMEDIATE_CROSSOVER: Final[str]
    BLEND_CROSSOVER: Final[str]
    SHUFFLE_CROSSOVER: Final[str]
    SIMULATED_BINARY_CROSSOVER: Final[str]
    GRAPH_CROSSOVER: Final[str]
    PARTIALLY_MAPPED_CROSSOVER: Final[str]
    EDGE_RECOMBINE_CROSSOVER: Final[str]

    UNIFORM_MUTATOR: Final[str]
    SCRAMBLE_MUTATOR: Final[str]
    SWAP_MUTATOR: Final[str]
    ARITHMETIC_MUTATOR: Final[str]
    GAUSSIAN_MUTATOR: Final[str]
    GRAPH_MUTATOR: Final[str]
    OPERATION_MUTATOR: Final[str]
    TREE_CROSSOVER: Final[str]
    HOIST_MUTATOR: Final[str]
    INVERSION_MUTATOR: Final[str]
    POLYNOMIAL_MUTATOR: Final[str]
    JITTER_MUTATOR: Final[str]

    UNIQUE_SCORE_FILTER: Final[str]

    OBJECTIVE: Final[str]
    MIN: Final[str]
    MAX: Final[str]

    ALL_EVENTS: Final[str]
    START_EVENT: Final[str]
    STOP_EVENT: Final[str]
    EPOCH_START_EVENT: Final[str]
    EPOCH_COMPLETE_EVENT: Final[str]
    ENGINE_IMPROVEMENT_EVENT: Final[str]

    SCORE_LIMIT: Final[str]
    GENERATIONS_LIMIT: Final[str]
    SECONDS_LIMIT: Final[str]
    CONVERGENCE_LIMIT: Final[str]
    METRIC_LIMIT: Final[str]
    EXPR_LIMIT: Final[str]

class _LossFunctions:
    MSE_LOSS: Final[str]
    MAE_LOSS: Final[str]
    CROSS_ENTROPY_LOSS: Final[str]
    DIFF_LOSS: Final[str]

components: _Components
loss_functions: _LossFunctions

class PyEngineInputType(StrEnum):
    Alterer = "Alterer"
    SurvivorSelector = "SurvivorSelector"
    OffspringSelector = "OffspringSelector"
    SpeciesThreshold = "SpeciesThreshold"
    TargetSpecies = "TargetSpecies"
    PopulationSize = "PopulationSize"
    OffspringFraction = "OffspringFraction"
    MaxPhenotypeAge = "MaxPhenotypeAge"
    MaxSpeciesAge = "MaxSpeciesAge"
    Objective = "Objective"
    Executor = "Executor"
    FrontRange = "FrontRange"
    Limit = "Limit"
    Diversity = "Diversity"
    Population = "Population"
    Subscriber = "Subscriber"
    Generation = "Generation"
    Checkpoint = "Checkpoint"
    Codec = "Codec"
    FitnessFunction = "FitnessFunction"
    Metric = "Metric"
    Filter = "Filter"
    Unknown = "Unknown"

class PyEngineControl:
    def pause(self) -> None: ...
    def resume(self) -> None: ...
    def stop(self) -> None: ...
    def step(self) -> None: ...

class PyGeneration:
    def index(self) -> int: ...
    def score(self) -> float: ...

class PyEngineInput:
    def __init__(
        self,
        input_type: PyEngineInputType,
        component: str | None,
        allowed_genes: set[str],
        args: dict[str, object] | None = None,
    ): ...

class PyEngineBuilder:
    def __init__(self, inputs: list[PyEngineInput]): ...
    def build(self) -> PyEngine: ...

class PyEngine:
    def __iter__(self) -> "PyEngine": ...
    def __next__(self) -> "PyGeneration": ...
    def run(self, options: list[PyEngineInput]) -> PyGeneration: ...

class PyFitnessFn[T]:
    @staticmethod
    def novelty_search(
        distance_fn: str,
        descriptor: Callable[[T], object],
        k: int,
        threshold: float,
        archive_size: int,
        is_batch: bool,
    ) -> PyFitnessFn: ...
    @staticmethod
    def custom(fitness_fn: Callable[[T], object], is_batch: bool) -> PyFitnessFn: ...
    @staticmethod
    def regression(
        features: list[list[float]],
        targets: list[list[float]],
        loss: str,
        is_batch: bool,
    ) -> PyFitnessFn: ...
