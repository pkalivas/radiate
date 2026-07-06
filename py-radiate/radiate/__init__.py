try:
    from .__version__ import __version__, __version_tuple__
except ImportError:
    __version__ = "unknown"
    __version_tuple__ = (0, 0, 0)

from ._dependancies import (
    _GIL_ENABLED,
    _NUMPY_AVAILABLE,
    _PANDAS_AVAILABLE,
    _POLARS_AVAILABLE,
    _TORCH_AVAILABLE,
)
from .codec import (
    BitCodec,
    CharCodec,
    FloatCodec,
    GraphCodec,
    IntCodec,
    PermutationCodec,
    TreeCodec,
)

# from .dsl import Filter
from .dtype import (
    Boolean,
    Char,
    Dict,
    Field,
    Float32,
    Float64,
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    List,
    Null,
    String,
    Struct,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    Usize,
)
from .engine import (
    CheckpointParam,
    Engine,
    EngineEvent,
    EventHandler,
    EventType,
    Front,
    FrontValue,
    Generation,
    LogParam,
    Metric,
    MetricCollector,
    MetricSet,
    Tag,
    UiParam,
    on_epoch,
    on_improvement,
    on_start,
    on_stop,
)
from .expr import Expr
from .fitness import (  # Base fitness classes,; Decorators; Loss functions
    MAE,
    MSE,
    BatchFitness,
    Diff,
    NoveltySearch,
    Regression,
    XEnt,
    fitness,
    novelty,
)
from .genome import (
    Chromosome,
    Ecosystem,
    Gene,
    Genotype,
    Phenotype,
    Population,
    Species,
    chromosome,
)
from .gp import AccuracyResult, Graph, Op, OpsConfig, Tree, accuracy
from .operators.alterer import (
    ArithmeticMutator,
    BlendCrossover,
    Cross,
    EdgeRecombinationCrossover,
    GaussianMutator,
    GraphCrossover,
    GraphMutator,
    HoistMutator,
    IntermediateCrossover,
    InversionMutator,
    JitterMutator,
    MeanCrossover,
    MultiPointCrossover,
    Mutate,
    OperationMutator,
    PartiallyMappedCrossover,
    PolynomialMutator,
    ScrambleMutator,
    ShuffleCrossover,
    SimulatedBinaryCrossover,
    SwapMutator,
    TreeCrossover,
    UniformCrossover,
    UniformMutator,
)
from .operators.distance import (
    CosineDistance,
    Dist,
    EuclideanDistance,
    HammingDistance,
    NeatDistance,
)
from .operators.executor import Executor
from .operators.filter import Filter
from .operators.limit import (
    ConvergenceLimit,
    GenerationsLimit,
    Limit,
    MetricLimit,
    ScoreLimit,
    SecondsLimit,
)
from .operators.selector import (
    BoltzmannSelector,
    EliteSelector,
    LinearRankSelector,
    NSGA2Selector,
    NSGA3Selector,
    RankSelector,
    RouletteSelector,
    Select,
    StochasticSamplingSelector,
    TournamentNSGA2Selector,
    TournamentSelector,
)
from .random import RandomProvider as random

MIN = "min"
MAX = "max"


__all__ = [
    # Version information
    "__version__",
    "__version_tuple__",
    # Dependencies
    "_GIL_ENABLED",
    "_PANDAS_AVAILABLE",
    "_POLARS_AVAILABLE",
    "_TORCH_AVAILABLE",
    "_NUMPY_AVAILABLE",
    # Random
    "random",
    # Expressions
    "Expr",
    # Codecs
    "PermutationCodec",
    "FloatCodec",
    "IntCodec",
    "CharCodec",
    "BitCodec",
    "GraphCodec",
    "TreeCodec",
    # Genome and Population
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
    "MetricCollector",
    "on_epoch",
    "on_improvement",
    "on_start",
    "on_stop",
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
    # Loss functions
    "MSE",
    "MAE",
    "XEnt",
    "Diff",
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
    "FrontValue",
    # Metrics
    "MetricSet",
    "Metric",
    "Tag",
    # Options
    "LogParam",
    "CheckpointParam",
    "UiParam",
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
    "Char",
    "Null",
    "List",
    "Dict",
    "Usize",
    # dsl
    "Select",
    "Dist",
    "Mutate",
    "Cross",
    "Limit",
    "Filter",
    # constants
    "MIN",
    "MAX",
]
