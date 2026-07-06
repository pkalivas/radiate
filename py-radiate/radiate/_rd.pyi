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
    def control(self) -> "PyEngineControl": ...
    def run(self, options: list[PyEngineInput]) -> PyGeneration: ...
