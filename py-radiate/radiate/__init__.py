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
    AnyCodec
)
from .random import RandomProvider as random
from .generation import Generation
from .genome import Gene, Chromosome, Genotype, Population, Phenotype
from .handlers import EventHandler, EventType
from .gp import Op, Graph, Tree

from .inputs.executor import Executor
from .inputs.problem import Regression, NoveltySearch
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
    InversionMutator
)

from .inputs.distance import HammingDistance, EuclideanDistance, NeatDistance, CosineDistance

from .inputs.limit import SecondsLimit, GenerationsLimit, ScoreLimit, ConvergenceLimit


__all__ = [
    # Version information
    "__version__",
    "__version_tuple__",
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
    "Gene",
    "Chromosome",
    "Genotype",
    "Population",
    "Phenotype",
    # GP
    "Tree",
    "Graph",
    "Op",
    # Handlers
    "EventHandler",
    "EventType",
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
]
