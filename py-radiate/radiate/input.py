from typing import List
from radiate.radiate import PyEngineInput, PyEngineInputType, PyGeneType
from .alterer import AlterBase

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
}

gene_type_mapping = {
    "float": PyGeneType.Float,
    "int": PyGeneType.Int,
    "bit": PyGeneType.Bit,
    "char": PyGeneType.Char,
    "graph": PyGeneType.Graph,
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


class EngineInput:
    def __init__(
        self,
        input_type: EngineInputType,
        component: str,
        allowed_genes: set[str],
        **kwargs,
    ):
        if input_type not in input_type_mapping:
            raise ValueError(f"Invalid input type: {input_type}")

        self._py_input = PyEngineInput(
            input_type=input_type_mapping[input_type],
            component=component,
            allowed_genes={gene_type_mapping[gt] for gt in allowed_genes if gt in gene_type_mapping},
            args={k: str(v) for k, v in kwargs.items()},
        )

    def __repr__(self):
        return self._py_input.__repr__()


class EngineBuilder:

    def __init__(self, gene_type: str):
        self._inputs = []
        self._gene_type = gene_type


    def set_alters(self, alters: List[AlterBase]):
        self._inputs.extend(alters)
        

    def __repr__(self):
        input_strs = ', \n'.join(repr(inp) for inp in self._inputs)
        return f"EngineBuilder(gene_type={self._gene_type}, inputs=[{input_strs}])"