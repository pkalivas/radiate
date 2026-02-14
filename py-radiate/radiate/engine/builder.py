from typing import Callable, Any
from dataclasses import dataclass

from radiate.radiate import PyEngine, PyEngineBuilder
from radiate.codec import CodecBase
from radiate.genome import Population, GeneType
from radiate.fitness import FitnessBase, CallableFitness
from radiate.operators import (
    AlterBase,
    DistanceBase,
    SelectorBase,
    TournamentSelector,
    RouletteSelector,
    LimitBase,
    Executor,
)

from .handlers import CallableEventHandler, EventHandler
from .generation import Generation

from radiate._bridge.input import EngineInput, EngineInputType
from radiate._typing import Subscriber, Decoding


@dataclass(slots=True)
class EngineConfig[G, T]:
    gene_type: GeneType | None = None
    codec: CodecBase[G, T] | None = None
    fitness_func: Callable[[T], Any] | FitnessBase[T] | None = None

    population: Population[G] | None = None
    offspring_selector: SelectorBase | None = None
    survivor_selector: SelectorBase | None = None
    alters: AlterBase | list[AlterBase] | None = None
    diversity: DistanceBase | None = None

    population_size: int = 100
    offspring_fraction: float = 0.8
    max_phenotype_age: int = 20
    max_species_age: int = 20
    species_threshold: float = 0.5

    objective: str | list[str] = "max"
    front_range: tuple[int, int] = (800, 900)
    executor: Executor | None = None
    subscribe: Subscriber | None = None
    generation: Generation[T] | None = None
    checkpoint_path: str | None = None


class EngineBuilder[G, T]:
    @classmethod
    def _default(cls, gene_type: GeneType, **kwargs) -> "EngineBuilder[G, T]":
        defaults = EngineConfig(**kwargs)
        inst = cls.__new__(cls)

        inst._inputs = []
        inst._gene_type = gene_type

        inst.set_population(defaults.population)
        inst.set_offspring_selector(
            defaults.offspring_selector or TournamentSelector(3)
        )
        inst.set_survivor_selector(defaults.survivor_selector or RouletteSelector())
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
        inst.set_checkpoint_path(defaults.checkpoint_path)
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
                component="codec",
                codec=codec.__backend__(),
            )
        )

        return self

    def set_fitness(
        self, fitness: FitnessBase | Callable[[Decoding[T]], Any] | None = None
    ):
        if fitness is None:
            return

        if isinstance(fitness, Callable):
            fitness = CallableFitness(fitness)

        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.FitnessFunction,
                component="fitnessfunction",
                fitness=fitness.__backend__(),
            )
        )

        return self

    def set_subscribers(self, subscriber: Subscriber | None):
        if subscriber is None:
            return

        def add_subscriber(sub: Subscriber):
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

    def set_generation(self, generation: Generation | None):
        if generation is None:
            return

        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.Generation,
                component="generation",
                generation=generation.__backend__(),
            )
        )

    def set_checkpoint_path(self, checkpoint_path: str | None):
        if checkpoint_path is None:
            return

        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.Checkpoint,
                component="checkpoint",
                path=checkpoint_path,
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

    def set_alters(self, alters: AlterBase | list[AlterBase] | None):
        if alters is None:
            return

        if isinstance(alters, AlterBase):
            alters = [alters]

        for alter in alters:
            self._inputs.append(
                EngineInput(
                    input_type=EngineInputType.Alterer,
                    component=alter.component,
                    allowed_genes=alter.allowed_genes,
                    rate=alter.rate,
                    **alter.args,
                )
            )

    def set_diversity(self, diversity: DistanceBase | None, species_threshold: float):
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

    def set_limits(self, limits: list[LimitBase] | None):
        if limits is None:
            return

        for limit in limits:
            self._inputs.append(
                EngineInput(
                    input_type=EngineInputType.Limit,
                    component=limit.component,
                    **limit.args,
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
                component="Objective",
                objective="|".join(objective),
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
                component="FrontRange",
                min=min,
                max=max,
            )
        )

    def set_executor(self, executor: Executor | None = None):
        if executor is None:
            executor = Executor.Serial()

        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.Executor,
                component=executor.component,
                **executor.args,
            )
        )
