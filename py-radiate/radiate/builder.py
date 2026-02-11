from typing import Callable, Any
from dataclasses import dataclass
from collections import defaultdict

from radiate.codec.base import CodecBase
from radiate.genome.population import Population
from radiate.fitness import FitnessBase
from radiate.handlers import CallableEventHandler, EventHandler
from radiate.radiate import PyEngine, PyEngineBuilder
from radiate.generation import Generation

from .inputs.limit import LimitBase
from .inputs.input import EngineInput, EngineInputType
from .inputs.selector import SelectorBase, TournamentSelector, RouletteSelector
from .inputs.alterer import AlterBase
from .inputs.distance import DistanceBase
from .inputs.executor import Executor
from .fitness import CallableFitness
from .genome import GeneType

from ._typing import Subscriber


@dataclass(slots=True)
class EngineConfig[G, T]:
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
    front_range: tuple[int, int] | None = (800, 900)
    executor: Executor | None = None
    subscribe: Subscriber | None = None
    generation: Generation[T] | None = None
    checkpoint_path: str | None = None

    def normalize(self) -> "EngineConfig[G, T]":
        if self.fitness_func is None:
            raise ValueError("A fitness function must be provided.")
        if not (0 < self.offspring_fraction <= 1):
            raise ValueError("offspring_fraction must be in (0, 1].")
        if self.population_size <= 0:
            raise ValueError("population_size must be > 0.")
        if self.max_phenotype_age <= 0 or self.max_species_age <= 0:
            raise ValueError("max ages must be > 0.")
        if self.survivor_selector is None:
            self.survivor_selector = TournamentSelector(k=3)
        if self.offspring_selector is None:
            self.offspring_selector = RouletteSelector()
        return self


class EngineBuilder:
    @classmethod
    def _default(cls, gene_type: GeneType) -> "EngineBuilder":
        instance = cls.__new__(cls)
        instance._gene_type = gene_type
        instance._inputs = []

        default_inputs = EngineConfig()
        instance.set_population(default_inputs.population)
        instance.set_offspring_selector(
            default_inputs.offspring_selector or TournamentSelector(3)
        )
        instance.set_survivor_selector(
            default_inputs.survivor_selector or RouletteSelector()
        )
        instance.set_alters(default_inputs.alters)
        instance.set_diversity(
            default_inputs.diversity, default_inputs.species_threshold
        )
        instance.set_population_size(default_inputs.population_size)
        instance.set_offspring_fraction(default_inputs.offspring_fraction)
        instance.set_max_age(default_inputs.max_phenotype_age)
        instance.set_max_species_age(default_inputs.max_species_age)
        instance.set_objective(default_inputs.objective)
        instance.set_front_range(*default_inputs.front_range)
        instance.set_executor(default_inputs.executor)
        instance.set_subscribers(default_inputs.subscribe)
        instance.set_generation(default_inputs.generation)
        instance.set_checkpoint_path(default_inputs.checkpoint_path)
        return instance

    @classmethod
    def from_config(cls, config: EngineConfig) -> "EngineBuilder":
        config = config.normalize()
        instance = cls._default(config.codec.gene_type)
        instance.set_codec(config.codec)
        instance.set_fitness(config.fitness_func)
        instance.set_population(config.population)
        instance.set_offspring_selector(config.offspring_selector)
        instance.set_survivor_selector(config.survivor_selector)
        instance.set_alters(config.alters)
        instance.set_diversity(config.diversity, config.species_threshold)
        instance.set_population_size(config.population_size)
        instance.set_offspring_fraction(config.offspring_fraction)
        instance.set_max_age(config.max_phenotype_age)
        instance.set_max_species_age(config.max_species_age)
        instance.set_objective(config.objective)
        if config.front_range is not None:
            instance.set_front_range(*config.front_range)
        instance.set_executor(config.executor)
        instance.set_subscribers(config.subscribe)
        instance.set_generation(config.generation)
        instance.set_checkpoint_path(config.checkpoint_path)
        return instance

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

    def set_codec(self, codec: CodecBase):
        if self._gene_type != codec.gene_type:
            raise ValueError(
                f"Codec {codec.component} does not support gene type {self._gene_type}"
            )

        self._inputs.append(
            EngineInput(
                input_type=EngineInputType.Codec,
                component="codec",
                codec=codec.__backend__(),
            )
        )

        return self

    def set_fitness(self, fitness: FitnessBase | Callable[[Any], Any]):
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
                    rate=alter.rate,
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

    def set_limits(self, limits: list[LimitBase]):
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
