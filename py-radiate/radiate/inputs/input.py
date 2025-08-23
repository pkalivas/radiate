from __future__ import annotations

from radiate.radiate import PyEngineInput, PyEngineInputType

from ..genome import (
    GENE_TYPE_MAPPING,
    GeneType,
)

input_type_mapping = {
    "Alterer": PyEngineInputType.Alterer,
    "SurvivorSelector": PyEngineInputType.SurvivorSelector,
    "OffspringSelector": PyEngineInputType.OffspringSelector,
    "SpeciesThreshold": PyEngineInputType.SpeciesThreshold,
    "PopulationSize": PyEngineInputType.PopulationSize,
    "OffspringFraction": PyEngineInputType.OffspringFraction,
    "MaxPhenotypeAge": PyEngineInputType.MaxPhenotypeAge,
    "MaxSpeciesAge": PyEngineInputType.MaxSpeciesAge,
    "Objective": PyEngineInputType.Objective,
    "Executor": PyEngineInputType.Executor,
    "FrontRange": PyEngineInputType.FrontRange,
    "Limit": PyEngineInputType.Limit,
    "Diversity": PyEngineInputType.Diversity,
    "Population": PyEngineInputType.Population,
    "Subscriber": PyEngineInputType.Subscriber,
}


class EngineInputType:
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


class EngineInput:
    def __init__(
        self,
        input_type: EngineInputType,
        component: str,
        allowed_genes: set[GeneType] | list[GeneType] | GeneType | None = None,
        **kwargs,
    ):
        if input_type not in input_type_mapping:
            raise ValueError(f"Invalid input type: {input_type}")

        if not allowed_genes:
            allowed_genes = GeneType.all()
        elif isinstance(allowed_genes, GeneType):
            allowed_genes = {allowed_genes}

        self._py_input = PyEngineInput(
            input_type=input_type_mapping[input_type],
            component=component,
            allowed_genes=set(
                [GENE_TYPE_MAPPING["rs"][gene_type] for gene_type in allowed_genes]
            ),
            args={k: v for k, v in kwargs.items()},
        )

    def py_input(self) -> PyEngineInput:
        return self._py_input

    def __repr__(self):
        return self._py_input.__repr__()
