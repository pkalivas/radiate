from typing import Callable, Any
from radiate.codec.base import CodecBase
from radiate.genome.population import Population
from radiate.fitness import FitnessBase
from radiate.handlers import CallableEventHandler, EventHandler
from radiate.radiate import PyEngine, PyEngineBuilder
from ._typing import Subscriber
from .inputs.input import EngineInput, EngineInputType
from .inputs.selector import SelectorBase
from .inputs.alterer import AlterBase
from .inputs.distance import DistanceBase
from .inputs.executor import Executor
from .fitness import CallableFitness
from .genome import GeneType


class EngineBuilder:
    def __init__(
        self,
        gene_type: GeneType,
        codec: CodecBase,
        problem: FitnessBase,
    ):
        self._inputs = []
        self._gene_type = gene_type
        self._codec = codec

        if isinstance(problem, Callable):
            self.fitness = CallableFitness(problem)
        else:
            self.fitness = problem

    def __repr__(self):
        input_strs = ", \n".join(repr(inp) for inp in self._inputs)
        return f"EngineBuilder(gene_type={self._gene_type}, inputs=[{input_strs}])"

    def build(self) -> PyEngine:
        """Build the PyEngine instance."""

        builder = PyEngineBuilder(
            codec=self._codec.codec,
            problem=self.fitness.problem,
            inputs=[self_input.__backend__() for self_input in self._inputs],
        )
        return builder.build()

    def inputs(self) -> list[EngineInput]:
        return self._inputs

    def set_subscribers(self, subscriber: Subscriber | None):
        if subscriber is None:
            return

        def add_subscriber(sub: EventHandler | Callable[[Any], None]):
            if isinstance(sub, EventHandler):
                self._inputs.append(
                    EngineInput(
                        input_type=EngineInputType.Subscriber,
                        component="subscriber",
                        subscriber=sub._py_handler,
                    )
                )
            elif isinstance(sub, Callable):
                self._inputs.append(
                    EngineInput(
                        input_type=EngineInputType.Subscriber,
                        component="subscriber",
                        subscriber=CallableEventHandler(sub)._py_handler,
                    )
                )
            else:
                raise TypeError(
                    "Subscriber list must contain only Callables or EventHandlers."
                )

        if isinstance(subscriber, list):
            for sub in subscriber:
                add_subscriber(sub)
        else:
            add_subscriber(subscriber)

    def set_population(self, population: Population):
        if population is None:
            return

        if not isinstance(population, Population):
            raise TypeError("population must be an instance of Population")
        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.Population,
                component="population",
                population=population.__backend__(),
            )
        )

    def set_survivor_selector(self, selector: SelectorBase):
        if self._gene_type not in selector.allowed_genes:
            raise ValueError(
                f"Selector {selector.component} does not support gene type {self._gene_type}"
            )

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
            raise ValueError(
                f"Selector {selector.component} does not support gene type {self._gene_type}"
            )

        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.OffspringSelector,
                component=selector.component,
                allowed_genes=selector.allowed_genes,
                **selector.args,
            )
        )

    def set_alters(self, alters: list[AlterBase] | None):
        if alters is None:
            return

        for alter in alters:
            self._inputs.append(
                EngineInput(
                    input_type=EngineInputType.Alterer,
                    component=alter.component,
                    allowed_genes=alter.allowed_genes,
                    **alter.args,
                )
            )

    def set_diversity(self, diversity: DistanceBase, species_threshold: float):
        if diversity is None:
            return

        if self._gene_type not in diversity.allowed_genes:
            raise ValueError(
                f"Diversity {diversity.component} does not support gene type {self._gene_type}"
            )

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

    def set_max_age(self, age: int):
        if age <= 0:
            raise ValueError("Max phenotype age must be greater than 0.")

        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.MaxPhenotypeAge,
                component="MaxPhenotypeAge",
                age=age,
            )
        )

    def set_max_species_age(self, age: int | None = None):
        if age is not None and age <= 0:
            raise ValueError("Max species age must be greater than 0.")

        if age is not None:
            self._inputs.append(
                EngineInput(
                    input_type=EngineInputType.MaxSpeciesAge,
                    component="MaxSpeciesAge",
                    age=age,
                )
            )

    def set_objective(
        self, objective: list[str] | str, front_range: tuple[int, int] | None = None
    ):
        if isinstance(objective, str):
            objective = [objective]
        if not all(obj in {"min", "max"} for obj in objective):
            raise ValueError("Objective must be 'min' or 'max'.")

        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.Objective,
                component="Objective",
                objective="|".join(objective),
            )
        )

        if front_range is not None and len(objective) > 1:
            self._inputs.append(
                EngineInput(
                    input_type=EngineInputType.FrontRange,
                    component="FrontRange",
                    min=front_range[0],
                    max=front_range[1],
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
