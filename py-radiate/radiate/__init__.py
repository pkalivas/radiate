try:
    from .__version__ import __version__, __version_tuple__
except ImportError:
    __version__ = "unknown"
    __version_tuple__ = (0, 0, 0)

from .engine import GeneticEngine
from .codec import (
    FloatCodec,
    IntCodec,
    CharCodec,
    BitCodec,
    GraphCodec,
    TreeCodec,
    PermutationCodec,
    AnyCodec,
)
from .random import RandomProvider as random
from .generation import Generation
from .genome import (
    gene,
    chromosome,
    Chromosome,
    Genotype,
    Population,
    Species,
    Ecosystem,
    Phenotype,
    Gene,
    AnyGene,
)
from .handlers import EventHandler, EventType, EngineEvent
from .gp import Op, Graph, Tree
from .metrics import MetricSet, Metric

from .inputs.executor import Executor
from .fitness import Regression, NoveltySearch, BatchFitness
from .inputs.selector import (
    TournamentSelector,
    RouletteSelector,
    RankSelector,
    EliteSelector,
    StochasticSamplingSelector,
    BoltzmannSelector,
    LinearRankSelector,
    NSGA2Selector,
    TournamentNSGA2Selector,
    SteadyStateSelector,
)

from .inputs.alterer import (
    BlendCrossover,
    IntermediateCrossover,
    ArithmeticMutator,
    UniformCrossover,
    UniformMutator,
    MultiPointCrossover,
    MeanCrossover,
    ShuffleCrossover,
    SimulatedBinaryCrossover,
    PartiallyMappedCrossover,
    GaussianMutator,
    ScrambleMutator,
    SwapMutator,
    GraphMutator,
    OperationMutator,
    GraphCrossover,
    TreeCrossover,
    HoistMutator,
    InversionMutator,
    PolynomialMutator,
    EdgeRecombinationCrossover,
    JitterMutator,
)

from .inputs.distance import (
    HammingDistance,
    EuclideanDistance,
    NeatDistance,
    CosineDistance,
)

from .inputs.limit import SecondsLimit, GenerationsLimit, ScoreLimit, ConvergenceLimit

from .dependancies import (
    _GIL_ENABLED,
    _NUMPY_AVAILABLE,
    _PANDAS_AVAILABLE,
    _POLARS_AVAILABLE,
)


__all__ = [
    # Version information
    "__version__",
    "__version_tuple__",
    # Dependencies
    "_NUMPY_AVAILABLE",
    "_GIL_ENABLED",
    "_PANDAS_AVAILABLE",
    "_POLARS_AVAILABLE",
    # Random
    "random",
    # Codecs
    "PermutationCodec",
    "FloatCodec",
    "IntCodec",
    "CharCodec",
    "BitCodec",
    "GraphCodec",
    "TreeCodec",
    "AnyCodec",
    # Genome and Population
    "gene",
    "chromosome",
    "Gene",
    "AnyGene",
    "Chromosome",
    "Genotype",
    "Phenotype",
    "Population",
    "Species",
    "Ecosystem",
    # GP
    "Tree",
    "Graph",
    "Op",
    # Handlers
    "EventHandler",
    "EventType",
    "EngineEvent",
    # Alters
    "BlendCrossover",
    "TreeCrossover",
    "GraphCrossover",
    "IntermediateCrossover",
    "MultiPointCrossover",
    "MeanCrossover",
    "ShuffleCrossover",
    "SimulatedBinaryCrossover",
    "PartiallyMappedCrossover",
    "UniformCrossover",
    "ArithmeticMutator",
    "UniformMutator",
    "GaussianMutator",
    "ScrambleMutator",
    "SwapMutator",
    "HoistMutator",
    "GraphMutator",
    "OperationMutator",
    "InversionMutator",
    "PolynomialMutator",
    "EdgeRecombinationCrossover",
    "JitterMutator",
    # Executor
    "Executor",
    # Limits
    "SecondsLimit",
    "GenerationsLimit",
    "ScoreLimit",
    "ConvergenceLimit",
    # Problem
    "Regression",
    "NoveltySearch",
    "BatchFitness",
    # Selectors
    "TournamentSelector",
    "RouletteSelector",
    "RankSelector",
    "EliteSelector",
    "StochasticSamplingSelector",
    "BoltzmannSelector",
    "LinearRankSelector",
    "NSGA2Selector",
    "TournamentNSGA2Selector",
    "SteadyStateSelector",
    # Diversity Measures
    "NeatDistance",
    "CosineDistance",
    "HammingDistance",
    "EuclideanDistance",
    # Engine
    "GeneticEngine",
    "Generation",
    # Metrics
    "MetricSet",
    "Metric",
]
