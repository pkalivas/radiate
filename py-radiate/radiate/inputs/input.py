from __future__ import annotations
from enum import Enum

from radiate.radiate import PyEngineInput, PyEngineInputType
from radiate.wrapper import PyObject

from ..genome import (
    GENE_TYPE_MAPPING,
    GeneType,
)


class EngineInputType(Enum):
    Alterer = "Alterer"
    SurvivorSelector = "SurvivorSelector"
    OffspringSelector = "OffspringSelector"
    SpeciesThreshold = "SpeciesThreshold"
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


input_type_mapping = {
    EngineInputType.Alterer: PyEngineInputType.Alterer,
    EngineInputType.SurvivorSelector: PyEngineInputType.SurvivorSelector,
    EngineInputType.OffspringSelector: PyEngineInputType.OffspringSelector,
    EngineInputType.SpeciesThreshold: PyEngineInputType.SpeciesThreshold,
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
}


class EngineInput(PyObject[PyEngineInput]):
    def __init__(
        self,
        input_type: EngineInputType,
        component: str,
        allowed_genes: set[GeneType] | list[GeneType] | GeneType | None = None,
        **kwargs,
    ):
        super().__init__()
        if input_type not in input_type_mapping:
            raise ValueError(f"Invalid input type: {input_type}")

        if not allowed_genes:
            allowed_genes = GeneType.all()
        elif isinstance(allowed_genes, GeneType):
            allowed_genes = {allowed_genes}

        self._pyobj = PyEngineInput(
            input_type=input_type_mapping[input_type],
            component=component,
            allowed_genes=set(
                GENE_TYPE_MAPPING["rs"][gene_type] for gene_type in allowed_genes
            ),
            args={k: v for k, v in kwargs.items()},
        )
