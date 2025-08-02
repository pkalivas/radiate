try:
    from .__version__ import __version__, __version_tuple__
except ImportError:
    __version__ = "unknown"
    __version_tuple__ = (0, 0, 0)

from .datatype import (
    DType,
    NumericType,
    IntegerType,
    FloatType,
    ArrayType,
    MatrixType,
    Int32,
    Int64,
    Float32,
    Float64,
    Bool,
    Int8Array,
    Int16Array,
    Int32Array,
    Int64Array,
    Float32Array,
    Float64Array,
    BoolArray,
    Int32Matrix,
    Int64Matrix,
    Float32Matrix,
    Float64Matrix,
)

from .engine import GeneticEngine
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
from .generation import Generation
from .genome import Gene, Chromosome, Genotype, Population, Phenotype
from .handlers import EventHandler, EventType
from .gp import Op

from .inputs.executor import Executor
from .fitness import Regression, NoveltySearch, fitness
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
)

from .inputs.distance import (
    HammingDistance,
    EuclideanDistance,
    NeatDistance,
    CosineDistance,
)

from .inputs.limit import SecondsLimit, GenerationsLimit, ScoreLimit, ConvergenceLimit

from .inputs.descriptor import PhenotypeDescriptor

from .dependancies import _NUMBA_AVAILABLE, _NUMPY_AVAILABLE

from radiate.radiate import (
    PyGraph as Graph,
    PyTree as Tree,
)


__all__ = [
    # Version information
    "__version__",
    "__version_tuple__",
    # Dependencies
    "_NUMBA_AVAILABLE",
    "_NUMPY_AVAILABLE",
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
    # Descriptors
    "PhenotypeDescriptor",
    # Fitness
    "fitness",
    # Data types
    "DType",
    "NumericType",
    "IntegerType",
    "FloatType",
    "ArrayType",
    "MatrixType",
    "Int32",
    "Int64",
    "Float32",
    "Float64",
    "Bool",
    "Int8Array",
    "Int16Array",
    "Int32Array",
    "Int64Array",
    "Float32Array",
    "Float64Array",
    "BoolArray",
    "Int32Matrix",
    "Int64Matrix",
    "Float32Matrix",
    "Float64Matrix",
]
