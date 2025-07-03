from typing import Optional
from radiate.radiate import PyEngineInput, PyEngineInputType, PyGeneType
from ..genome.gene import GeneType

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
}

gene_type_mapping = {
    "float": PyGeneType.Float,
    "int": PyGeneType.Int,
    "bit": PyGeneType.Bit,
    "char": PyGeneType.Char,
    'permutation': PyGeneType.Permutation,
    "graph": PyGeneType.Graph,
    'tree': PyGeneType.Tree,
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


class EngineInput:
    def __init__(
        self,
        input_type: EngineInputType,
        component: str,
        allowed_genes: Optional[set[str]] = None,
        **kwargs,
    ):
        if input_type not in input_type_mapping:
            raise ValueError(f"Invalid input type: {input_type}")
        if not allowed_genes:
            allowed_genes = GeneType.ALL

        self._py_input = PyEngineInput(
            input_type=input_type_mapping[input_type],
            component=component,
            allowed_genes={
                gene_type_mapping[gt] for gt in allowed_genes if gt in gene_type_mapping
            },
            args={k: v for k, v in kwargs.items()},
        )

    def py_input(self) -> PyEngineInput:
        return self._py_input

    def __repr__(self):
        return self._py_input.__repr__()
