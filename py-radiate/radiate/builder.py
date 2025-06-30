from typing import List, Optional, Tuple, Callable
from radiate.codec.codec import CodecBase
from radiate.inputs.problem import ProblemBase
from radiate.radiate import PyEngine, PyEngineBuilder
from .inputs.input import EngineInput, EngineInputType
from .inputs.selector import SelectorBase
from .inputs.alterer import AlterBase
from .inputs.diversity import DiversityBase 
from .inputs.executor import Executor
from .inputs.problem import CallableProblem

class EngineBuilder:

    def __init__(self, gene_type: str, codec: CodecBase, problem: ProblemBase):
        self._inputs = []
        self._gene_type = gene_type
        self._codec = codec

        if isinstance(problem, Callable):
            self.problem = CallableProblem(problem)
        else:
            print("Using provided problem instance")
            self.problem = problem

    def build(self) -> PyEngine:
        builder = PyEngineBuilder(
            gene_type=self._gene_type,
            codec=self._codec.codec,
            problem=self.problem.problem,
            inputs=[self_input.py_input() for self_input in self._inputs],
        )
        return builder.build()

    def inputs(self) -> List[EngineInput]:
        return self._inputs

    def set_survivor_selector(self, selector: SelectorBase):
        if self._gene_type not in selector.allowed_genes:
            raise ValueError(f"Selector {selector.component} does not support gene type {self._gene_type}")
        
        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.SurvivorSelector,
                component=selector.component,
                allowed_genes=selector.allowed_genes,
                **selector.args,
            )
        )

    def set_offspring_selector(self, selector: SelectorBase):
        if self._gene_type not in selector.allowed_genes:
            raise ValueError(f"Selector {selector.component} does not support gene type {self._gene_type}")
        
        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.OffspringSelector,
                component=selector.component,
                allowed_genes=selector.allowed_genes,
                **selector.args,
            )
        )

    def set_alters(self, alters: List[AlterBase]):
        for alter in alters:
            if self._gene_type not in alter.allowed_genes:
                raise ValueError(f"Alterer {alter.component} does not support gene type {self._gene_type}")

            self._inputs.append(
                EngineInput(
                input_type=EngineInputType.Alterer,
                component=alter.component,
                allowed_genes=alter.allowed_genes,
                **alter.args,
            )
        )

    def set_diversity(self, diversity: DiversityBase, species_threshold: float, max_species_age: int):
        if diversity is None:
            return
        
        if self._gene_type not in diversity.allowed_genes:
            raise ValueError(f"Diversity {diversity.component} does not support gene type {self._gene_type}")

        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.Diversity,
                component=diversity.component,
                allowed_genes=diversity.allowed_genes,
                **diversity.args,
            )
        ) 

        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.SpeciesThreshold,
                component="SpeciesThreshold",
                allowed_genes=diversity.allowed_genes,
                threshold=species_threshold,
                max_species_age=max_species_age,
            )
        )

    def set_population_size(self, size: int):
        if size <= 0:
            raise ValueError("Population size must be greater than 0.")
        
        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.PopulationSize,
                component="PopulationSize",
                size=size,
            )
        )

    def set_offspring_fraction(self, fraction: float):
        if not (0.0 < fraction <= 1.0):
            raise ValueError("Offspring fraction must be in the range (0.0, 1.0].")
        
        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.OffspringFraction,
                component="OffspringFraction",
                fraction=fraction,
            )
        )

    def set_max_phenotype_age(self, age: int):
        if age <= 0:
            raise ValueError("Max phenotype age must be greater than 0.")

        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.MaxPhenotypeAge,
                component="MaxPhenotypeAge",
                age=age,
            )
        )

    def set_objective(self, objective: List[str] | str, front_range: Optional[Tuple[int, int]] = None):
        if isinstance(objective, str):
            objective = [objective]
        if not all(obj in {"min", "max"} for obj in objective):
            raise ValueError("Objective must be 'min' or 'max'.")
        
        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.Objective,
                component="Objective",
                objective='|'.join(objective)
            )
        )

        if front_range is not None and len(objective) > 1:
            self._inputs.append(
                EngineInput(
                    input_type=EngineInputType.FrontRange,
                    component="FrontRange",
                    front_min=front_range[0],
                    front_max=front_range[1],
                )
            )

    def set_executor(self, executor: Executor):
        if executor is None:
            executor = Executor.Serial()

        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.Executor,
                component=executor.component,
                **executor.args,
            )
        )

    def __repr__(self):
        input_strs = ', \n'.join(repr(inp) for inp in self._inputs)
        return f"EngineBuilder(gene_type={self._gene_type}, inputs=[{input_strs}])"