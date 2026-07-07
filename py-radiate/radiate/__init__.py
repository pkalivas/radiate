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
from .dsl.dtype import (
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
from .dsl.expr import Expr
from .dsl.loss import (
    MAE,
    MSE,
    Diff,
    XEnt,
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
    Cross,
    Mutate,
)
from .operators.distance import Dist
from .operators.executor import Executor
from .operators.filter import Filter
from .operators.fitness import Fitness, fitness, novelty
from .operators.limit import Limit
from .operators.selector import Select
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
    "Cross",
    "Mutate",
    # Executor
    "Executor",
    # Limits
    "Limit",
    # Filters
    "Filter",
    # Fitness
    "Fitness",
    # Problem
    "fitness",
    "novelty",
    # Loss functions
    "MSE",
    "MAE",
    "XEnt",
    "Diff",
    # Selectors
    "Select",
    # Diversity Measures
    "Dist",
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
    # constants
    "MIN",
    "MAX",
]
