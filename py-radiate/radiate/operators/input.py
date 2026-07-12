from __future__ import annotations

from enum import StrEnum
from typing import override

from .._bridge import RsObject
from .._rd import PyEngineInput, PyEngineInputType
from ..genome import GENE_TYPE_MAPPING, GeneType


class EngineInputType(StrEnum):
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


input_type_mapping = {
    EngineInputType.Alterer: PyEngineInputType.Alterer,
    EngineInputType.SurvivorSelector: PyEngineInputType.SurvivorSelector,
    EngineInputType.OffspringSelector: PyEngineInputType.OffspringSelector,
    EngineInputType.SpeciesThreshold: PyEngineInputType.SpeciesThreshold,
    EngineInputType.TargetSpecies: PyEngineInputType.TargetSpecies,
    EngineInputType.PopulationSize: PyEngineInputType.PopulationSize,
    EngineInputType.OffspringFraction: PyEngineInputType.OffspringFraction,
    EngineInputType.MaxPhenotypeAge: PyEngineInputType.MaxPhenotypeAge,
    EngineInputType.MaxSpeciesAge: PyEngineInputType.MaxSpeciesAge,
    EngineInputType.Objective: PyEngineInputType.Objective,
    EngineInputType.Executor: PyEngineInputType.Executor,
    EngineInputType.FrontRange: PyEngineInputType.FrontRange,
    EngineInputType.Limit: PyEngineInputType.Limit,
    EngineInputType.Diversity: PyEngineInputType.Diversity,
    EngineInputType.Population: PyEngineInputType.Population,
    EngineInputType.Subscriber: PyEngineInputType.Subscriber,
    EngineInputType.Generation: PyEngineInputType.Generation,
    EngineInputType.Checkpoint: PyEngineInputType.Checkpoint,
    EngineInputType.Codec: PyEngineInputType.Codec,
    EngineInputType.FitnessFunction: PyEngineInputType.FitnessFunction,
    EngineInputType.Metric: PyEngineInputType.Metric,
    EngineInputType.Filter: PyEngineInputType.Filter,
    EngineInputType.Unknown: PyEngineInputType.Unknown,
}


class EngineInput(RsObject):
    _input_type: EngineInputType
    _component: str | None
    _allowed_genes: set[GeneType] | list[GeneType] | GeneType | None
    _args: dict[str, object] | None

    def __init__(
        self,
        input_type: EngineInputType,
        component: str | None = None,
        allowed_genes: set[GeneType] | list[GeneType] | GeneType | None = None,
        **kwargs,
    ):
        self._input_type = input_type
        self._component = component
        self._allowed_genes = allowed_genes
        self._args = {**kwargs} if kwargs else None

    @property
    def component(self) -> str | None:
        return self._component

    @property
    def input_type(self) -> EngineInputType:
        if not input_type_mapping.get(self._input_type):
            raise ValueError(f"Invalid input type: {self._input_type}")
        return self._input_type

    @property
    def allowed_genes(self) -> set[GeneType]:
        if isinstance(self._allowed_genes, GeneType):
            return {self._allowed_genes}
        elif isinstance(self._allowed_genes, list):
            return set(self._allowed_genes)
        elif isinstance(self._allowed_genes, set):
            return self._allowed_genes
        else:
            return GeneType.all()

    @property
    def args(self) -> dict[str, object]:
        return self._args if self._args is not None else {}

    @override
    def __backend__(self) -> PyEngineInput:
        return PyEngineInput(
            input_type=input_type_mapping[self._input_type],
            component=self._component,
            allowed_genes=set(
                GENE_TYPE_MAPPING["rs"][gene_type] for gene_type in self.allowed_genes
            ),
            args={
                name: value.__backend__() if isinstance(value, RsObject) else value
                for name, value in self.args.items()
            },
        )
