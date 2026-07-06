from __future__ import annotations

from enum import StrEnum
from typing import override

from .._bridge.wrapper import RsObject
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
    _component: str
    _input_type: EngineInputType
    _allowed_genes: set[GeneType] | list[GeneType] | GeneType | None
    _args: dict[str, object] | None

    def __init__(
        self,
        component: str,
        input_type: EngineInputType,
        allowed_genes: set[GeneType] | list[GeneType] | GeneType | None = None,
        **kwargs,
    ):
        self._component = component
        self._input_type = input_type
        self._allowed_genes = allowed_genes
        self._args = {**kwargs} if kwargs else None

    @property
    def component(self) -> str:
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

        # super().__init__()
        # if input_type not in input_type_mapping:
        #     raise ValueError(f"Invalid input type: {input_type}")

        # if not allowed_genes:
        #     allowed_genes = GeneType.all()
        # elif isinstance(allowed_genes, GeneType):
        #     allowed_genes = {allowed_genes}

        # args = {
        #     name: value.__backend__() if isinstance(value, RsObject) else value
        #     for name, value in kwargs.items()
        # }

        # self._pyobj = PyEngineInput(
        #     input_type=input_type_mapping[input_type],
        #     component=component,
        #     allowed_genes=set(
        #         GENE_TYPE_MAPPING["rs"][gene_type] for gene_type in allowed_genes
        #     ),
        #     args=args,
        # )

    # def __str__(self) -> str:
    #     return self.__backend__().__str__()
