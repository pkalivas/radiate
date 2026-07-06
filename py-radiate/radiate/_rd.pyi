from enum import StrEnum

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

class PyAltererComponentType(StrEnum):
    MultiPointCrossover = "MultiPointCrossover"
    UniformCrossover = "UniformCrossover"
    MeanCrossover = "MeanCrossover"
    IntermediateCrossover = "IntermediateCrossover"
    BlendCrossover = "BlendCrossover"
    ShuffleCrossover = "ShuffleCrossover"
    SimulatedBinaryCrossover = "SimulatedBinaryCrossover"
    GraphCrossover = "GraphCrossover"
    TreeCrossover = "TreeCrossover"
    PartiallyMappedCrossover = "PartiallyMappedCrossover"
    EdgeRecombinationCrossover = "EdgeRecombinationCrossover"
    UniformMutator = "UniformMutator"
    ScrambleMutator = "ScrambleMutator"
    SwapMutator = "SwapMutator"
    ArithmeticMutator = "ArithmeticMutator"
    GaussianMutator = "GaussianMutator"
    GraphMutator = "GraphMutator"
    OperationMutator = "OperationMutator"
    HoistMutator = "HoistMutator"
    InversionMutator = "InversionMutator"
    PolynomialMutator = "PolynomialMutator"
    JitterMutator = "JitterMutator"

class PyLimitComponentType(StrEnum):
    Generations = "Generations"
    Seconds = "Seconds"
    Score = "Score"
    Convergence = "Convergence"
    Metric = "Metric"
    Expr = "Expr"

class PyDistanceComponentType(StrEnum):
    Hamming = "Hamming"
    Euclidean = "Euclidean"
    Cosine = "Cosine"
    Neat = "Neat"

class PyFilterComponentType(StrEnum):
    UniqueScore = "UniqueScore"

class PyExecutorComponentType(StrEnum):
    Serial = "Serial"
    FixedSizedWorkerPool = "FixedSizedWorkerPool"
    WorkerPool = "WorkerPool"

class PySelectorComponentType(StrEnum):
    Tournament = "Tournament"
    RouletteWheel = "RouletteWheel"
    Rank = "Rank"
    StochasticUniversal = "StochasticUniversal"
    Boltzmann = "Boltzmann"
    Elite = "Elite"
    Random = "Random"
    NSGA2 = "NSGA2"
    NSGA3 = "NSGA3"
    TournamentNSGA2 = "TournamentNSGA2"
    LinearRank = "LinearRank"

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
        component: str,
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
