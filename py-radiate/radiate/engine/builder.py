from dataclasses import dataclass
from typing import Any, Callable

from .._rd import PyEngine, PyEngineBuilder
from .._typing import Subscriber
from ..codec import CodecBase
from ..dsl.expr import Expr
from ..fitness import CallableFitness, FitnessBase
from ..genome import GeneType, Population
from ..operators import (
    AlterBase,
    Executor,
    Fitness,
)
from ..operators.distance import Dist
from ..operators.filter import Filter
from ..operators.input import EngineInput, EngineInputType
from ..operators.limit import Limit
from ..operators.selector import Select
from .generation import Generation
from .handlers import CallableEventHandler, EventHandler


@dataclass(slots=True)
class EngineConfig[G, T]:
    gene_type: GeneType | None = None
    codec: CodecBase[G, T] | None = None
    fitness_func: Callable[[T], Any] | Fitness | None = None

    population: Population[G] | None = None
    offspring_selector: Select | None = None
    survivor_selector: Select | None = None
    alters: AlterBase | list[AlterBase] | None = None
    diversity: Dist | None = None

    population_size: int = 100
    offspring_fraction: float = 0.8
    max_phenotype_age: int = 20
    max_species_age: int = 20
    species_threshold: Expr | float = 0.5

    objective: str | list[str] = "max"
    front_range: tuple[int, int] = (800, 900)
    executor: Executor | None = None
    subscribe: Subscriber | None = None
    generation: Generation[G, T] | None = None
    checkpoint_path: str | None = None


class EngineBuilder[G, T]:
    @classmethod
    def _default(cls, gene_type: GeneType, **kwargs) -> "EngineBuilder[G, T]":
        defaults = EngineConfig(**kwargs)
        inst = cls.__new__(cls)

        inst._inputs = []
        inst._gene_type = gene_type

        inst.set_population(defaults.population)
        inst.set_offspring_selector(defaults.offspring_selector or Select.tournament(3))
        inst.set_survivor_selector(defaults.survivor_selector or Select.roulette())
        inst.set_alters(defaults.alters)
        inst.set_diversity(defaults.diversity, defaults.species_threshold)
        inst.set_population_size(defaults.population_size)
        inst.set_offspring_fraction(defaults.offspring_fraction)
        inst.set_max_age(defaults.max_phenotype_age)
        inst.set_max_species_age(defaults.max_species_age)
        inst.set_objective(defaults.objective)
        inst.set_front_range(*defaults.front_range)
        inst.set_executor(defaults.executor)
        inst.set_subscribers(defaults.subscribe)
        inst.set_generation(defaults.generation)
        inst.set_checkpoint_path(defaults.checkpoint_path, ignore_not_found=True)
        inst.set_fitness(defaults.fitness_func)
        inst.set_codec(defaults.codec)

        return inst

    def __init__(self, gene_type: GeneType):
        self._inputs = []
        self._gene_type = gene_type

    def __repr__(self):
        input_strs = ", \n".join(repr(inp) for inp in self._inputs)
        return f"EngineBuilder(gene_type={self._gene_type}, inputs=[{input_strs}])"

    def build(self) -> PyEngine:
        """Build the PyEngine instance."""
        builder = PyEngineBuilder(
            inputs=[self_input.__backend__() for self_input in self._inputs],
        )

        return builder.build()

    def inputs(self) -> list[EngineInput]:
        return self._inputs

    def set_codec(self, codec: CodecBase[G, T] | None = None):
        if codec is None:
            return

        if self._gene_type != codec.gene_type:
            raise ValueError(
                f"Codec {codec} does not support gene type {self._gene_type}"
            )

        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.Codec,
                codec=codec.__backend__(),
            )
        )

        return self

    def set_fitness(self, fitness: Fitness | Callable[[T], Any] | None = None):
        if fitness is None:
            return

        if isinstance(fitness, Callable):
            fitness = Fitness.custom(fitness, is_batch=False)

        self._inputs.append(fitness)

        return self

    def set_subscribers(self, subscriber: Subscriber | list[Subscriber] | None):
        if subscriber is None:
            return

        def add_subscriber(sub: Subscriber):
            if isinstance(sub, EventHandler):
                self._inputs.append(
                    EngineInput(
                        input_type=EngineInputType.Subscriber,
                        subscriber=sub._py_handler,
                    )
                )
            elif isinstance(sub, Callable):
                self._inputs.append(
                    EngineInput(
                        input_type=EngineInputType.Subscriber,
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

    def set_generation(self, generation: Generation | None):
        if generation is None:
            return

        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.Generation,
                generation=generation.__backend__(),
            )
        )

    def set_checkpoint_path(self, checkpoint_path: str | None, ignore_not_found: bool):
        if checkpoint_path is None:
            return

        file_type = checkpoint_path.split(".")[-1].lower()
        if file_type not in {"pkl", "json"}:
            raise ValueError(
                "Checkpoint file type must be 'pkl' or 'json'. "
                f"Got '{file_type}' from path '{checkpoint_path}'."
            )

        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.Checkpoint,
                path=checkpoint_path,
                ignore_not_found=ignore_not_found,
                file_type=file_type,
            )
        )

    def set_population(self, population: Population[G] | None):
        if population is None:
            return

        if not isinstance(population, Population):
            raise TypeError("population must be an instance of Population")
        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.Population,
                population=population.__backend__(),
            )
        )

    def set_metrics(self, metrics: dict[str, Expr] | None):
        if metrics is None:
            return

        for name, expr in metrics.items():
            self._inputs.append(
                EngineInput(
                    input_type=EngineInputType.Metric,
                    name=name,
                    expr=expr.__backend__(),
                )
            )

    def set_survivor_selector(self, selector: Select):
        if self._gene_type not in selector.allowed_genes:
            raise ValueError(
                f"Selector {selector.component} does not support gene type {self._gene_type}"
            )

        self._inputs.append(selector.to_survivor_selector())

    def set_offspring_selector(self, selector: Select):
        if self._gene_type not in selector.allowed_genes:
            raise ValueError(
                f"Selector {selector.component} does not support gene type {self._gene_type}"
            )

        self._inputs.append(selector.to_offspring_selector())

    def set_alters(self, alters: AlterBase | list[AlterBase] | None):
        if alters is None:
            return

        if isinstance(alters, AlterBase):
            alters = [alters]

        for alter in alters:
            self._inputs.append(alter)

    def set_diversity(
        self,
        diversity: Dist | None,
        species_threshold: Expr | float,
        target_species: int | None = None,
    ):
        if diversity is None:
            return

        if self._gene_type not in diversity.allowed_genes:
            raise ValueError(
                f"Diversity {diversity.component} does not support gene type {self._gene_type}"
            )

        self._inputs.append(diversity)

        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.SpeciesThreshold,
                threshold=species_threshold,
            )
        )

        if target_species is not None:
            if not isinstance(target_species, (int, float)):
                raise TypeError(
                    "Target species must be an int or float, "
                    f"got {type(target_species).__name__}"
                )

            if target_species <= 0:
                raise ValueError("Target species must be greater than 0.")

            self._inputs.append(
                EngineInput(
                    input_type=EngineInputType.TargetSpecies,
                    target_species=int(target_species),
                )
            )

    def set_limits(self, limits: list[Limit] | None):
        if limits is None:
            return

        for limit in limits:
            self._inputs.append(limit)

    def set_filters(self, filters: list[Filter] | None):
        if filters is None:
            return

        for filter in filters:
            self._inputs.append(filter)

    def set_population_size(self, size: int):
        if size <= 0:
            raise ValueError("Population size must be greater than 0.")

        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.PopulationSize,
                size=size,
            )
        )

    def set_offspring_fraction(self, fraction: float):
        if not (0.0 < fraction <= 1.0):
            raise ValueError("Offspring fraction must be in the range (0.0, 1.0].")

        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.OffspringFraction,
                fraction=fraction,
            )
        )

    def set_max_age(self, age: int):
        if age <= 0:
            raise ValueError("Max phenotype age must be greater than 0.")

        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.MaxPhenotypeAge,
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
                    age=age,
                )
            )

    def set_objective(
        self,
        objective: list[str] | str,
    ):
        if isinstance(objective, str):
            objective = [objective]
        if not all(obj in {"min", "max"} for obj in objective):
            raise ValueError("Objective must be 'min' or 'max'.")

        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.Objective,
                objective=objective,
            )
        )

    def set_front_range(self, min: int, max: int):
        if min < 0 or max < 0:
            raise ValueError("Front range values must be non-negative.")
        if min >= max:
            raise ValueError("Front range min must be less than max.")

        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.FrontRange,
                min=min,
                max=max,
            )
        )

    def set_executor(self, executor: Executor | None = None):
        if executor is None:
            executor = Executor.Serial()

        self._inputs.append(executor)
