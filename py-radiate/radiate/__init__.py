try:
    from .__version__ import __version__, __version_tuple__
except ImportError:
    __version__ = "unknown"
    __version_tuple__ = (0, 0, 0)

from .engine import Engine
from .codec import (
    FloatCodec,
    IntCodec,
    CharCodec,
    BitCodec,
    GraphCodec,
    TreeCodec,
    PermutationCodec,
)
from .random import RandomProvider as random
from .front import Front
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
)
from .handlers import EventHandler, EventType, EngineEvent
from .gp import Op, Graph, Tree, accuracy, OpsConfig, AccuracyResult
from .metrics import MetricSet, Metric, Tag

from .inputs.executor import Executor
from .fitness import Regression, NoveltySearch, BatchFitness, fitness, novelty
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
    NSGA3Selector,
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

from .inputs.limit import (
    SecondsLimit,
    GenerationsLimit,
    ScoreLimit,
    ConvergenceLimit,
    MetricLimit,
)

from .inputs.rate import Rate
from .inputs import rate

from .option import EngineLog, EngineCheckpoint, EngineUi

from .dtype import (
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    Float32,
    Float64,
    Boolean,
    Struct,
    List,
    Field,
    String,
    Null,
    Op32,
    Node,
)

from .dsl import Select, Mutate, Cross, Dist, Limit as lim

from ._dependancies import (
    _GIL_ENABLED,
    _NUMPY_AVAILABLE,
    _PANDAS_AVAILABLE,
    _POLARS_AVAILABLE,
)

MIN = "min"
MAX = "max"


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
    # Genome and Population
    "gene",
    "chromosome",
    "Gene",
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
    "accuracy",
    "OpsConfig",
    "AccuracyResult",
    # Handlers
    "EventHandler",
    "EventType",
    "EngineEvent",
    # Rate
    "Rate",
    "rate",
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
    "MetricLimit",
    # Problem
    "Regression",
    "NoveltySearch",
    "BatchFitness",
    "fitness",
    "novelty",
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
    "NSGA3Selector",
    # Diversity Measures
    "NeatDistance",
    "CosineDistance",
    "HammingDistance",
    "EuclideanDistance",
    # Engine
    "Engine",
    "Generation",
    # Front,
    "Front",
    # Metrics
    "MetricSet",
    "Metric",
    "Tag",
    # Options
    "EngineLog",
    "EngineCheckpoint",
    "EngineUi",
    # Dtype
    "UInt8",
    "UInt16",
    "UInt32",
    "UInt64",
    "UInt128",
    "Int8",
    "Int16",
    "Int32",
    "Int64",
    "Int128",
    "Float32",
    "Float64",
    "Boolean",
    "Struct",
    "Field",
    "String",
    "Null",
    "List",
    "Op32",
    "Node",
    # dsl
    "Select",
    "Dist",
    "Mutate",
    "Cross",
    "lim",
    
    # constants
    "MIN",
    "MAX",
    # "Limit",
]


