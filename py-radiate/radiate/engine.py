from typing import Any, Callable, List
from .selector import Selector, TournamentSelector, RouletteSelector
from .alterer import Alterer, UniformCrossover, UniformMutator
from ._typing import GeneType, ObjectiveType
from .codex import FloatCodex, IntCodex, CharCodex
from .limit import Limit

from radiate.radiate import (
    PyEngineBuilder,
    PyFloatEngine,
    PyIntEngine,
    PyGeneration,
    PyCharEngine,
)


class GeneticEngine:
    """
    Genetic Engine for optimization problems.
    This class serves as the main interface for running genetic algorithms, allowing
    the customization of various parameters of the engine.
    """

    def __init__(
        self,
        codex: FloatCodex | IntCodex | CharCodex,
        fitness_func: Callable[[Any], Any],
        offspring_selector: Selector = None,
        survivor_selector: Selector = None,
        alters: None | Alterer | List[Alterer] = None,
        population_size: int = 100,
        offspring_fraction: float = 0.8,
        objectives: str | List[str] = ObjectiveType.MIN,
        num_threads: int = 1,
    ):
        self.codex = codex
        self.fitness_func = fitness_func

        if isinstance(self.codex, FloatCodex):
            self.gene_type = GeneType.FLOAT
        elif isinstance(self.codex, IntCodex):
            self.gene_type = GeneType.INT
        elif isinstance(self.codex, CharCodex):
            self.gene_type = GeneType.CHAR
        else:
            raise TypeError(f"Codex type {type(self.codex)} is not supported.")

        survivor_selector = self.__get_params(
            survivor_selector or TournamentSelector(k=3)
        )
        offspring_selector = self.__get_params(offspring_selector or RouletteSelector())
        alters = self.__get_params(alters or [UniformCrossover(), UniformMutator()])
        objectives = self.__get_objectives(objectives)

        self.builder = PyEngineBuilder(
            objectives=objectives,
            survivor_selector=survivor_selector,
            offspring_selector=offspring_selector,
            alters=alters,
            population_size=population_size,
            offspring_fraction=offspring_fraction,
            num_threads=num_threads,
        )

    def run(self, limits: Limit | List[Limit], log: bool = False) -> PyGeneration:
        """Run the engine with the given limits."""
        if limits is None:
            raise ValueError("Limits must be provided.")
        limits = [
            lim.params for lim in (limits if isinstance(limits, list) else [limits])
        ]
        engine = self.__get_engine()
        return engine.run(limits, log)

    def population_size(self, size: int):
        """Set the population size."""
        if size <= 0:
            raise ValueError("Population size must be greater than 0.")
        self.builder.set_population_size(size)

    def survivor_selector(self, selector: Selector):
        """Set the survivor selector."""
        if selector is None:
            raise ValueError("Selector must be provided.")
        self.builder.set_survivor_selector(self.__get_params(selector))

    def offspring_selector(self, selector: Selector):
        """Set the offspring selector."""
        if selector is None:
            raise ValueError("Selector must be provided.")
        self.builder.set_offspring_selector(self.__get_params(selector))

    def alters(self, alters: Alterer | List[Alterer]):
        """Set the alters."""
        if alters is None:
            raise ValueError("Alters must be provided.")
        self.builder.set_alters(self.__get_params(alters))

    def offspring_fraction(self, fraction: float):
        """Set the offspring fraction."""
        if not (0 < fraction <= 1):
            raise ValueError("Offspring fraction must be between 0 and 1.")
        self.builder.set_offspring_fraction(fraction)

    def minimizing(self):
        """Set the objectives."""
        self.builder.set_objectives(self.__get_objectives(ObjectiveType.MIN))

    def maximizing(self):
        """Set the objectives."""
        self.builder.set_objectives(self.__get_objectives(ObjectiveType.MAX))

    def num_threads(self, num_threads: int):
        """Set the number of threads."""
        if num_threads <= 0:
            raise ValueError("Number of threads must be greater than 0.")
        self.builder.set_num_threads(num_threads)

    def __get_engine(self):
        """Get the engine."""
        if self.gene_type == GeneType.FLOAT:
            return PyFloatEngine(self.codex.codex, self.fitness_func, self.builder)
        elif self.gene_type == GeneType.INT:
            return PyIntEngine(self.codex.codex, self.fitness_func, self.builder)
        elif self.gene_type == GeneType.CHAR:
            return PyCharEngine(self.codex.codex, self.fitness_func, self.builder)
        else:
            raise TypeError(f"Gene type {self.gene_type} is not supported.")

    def __get_objectives(self, objectives: str | List[str]) -> List[str]:
        """Get the objectives."""
        if objectives is None:
            raise ValueError("Objectives must be provided.")
        if isinstance(objectives, str):
            if objectives not in [ObjectiveType.MIN, ObjectiveType.MAX]:
                raise ValueError("Objectives must be 'min' or 'max'.")
            return [objectives]
        if isinstance(objectives, list):
            for obj in objectives:
                if obj not in [ObjectiveType.MIN, ObjectiveType.MAX]:
                    raise ValueError("Objectives must be 'min' or 'max'.")
            return objectives
        raise TypeError(f"Objectives type {type(objectives)} is not supported.")

    def __get_params(self, value):
        if isinstance(value, Selector):
            return value.params
        if isinstance(value, Alterer):
            return [value.params]
        if isinstance(value, list):
            if all(isinstance(alter, Alterer) for alter in value):
                for alter in value:
                    if not alter.is_valid(self.gene_type):
                        raise TypeError(
                            f"Alterer {alter} is not valid for genome type {GeneType.FLOAT}."
                        )
                return [alter.params for alter in value]
        raise TypeError(f"Param type {type(value)} is not supported.")

    def __repr__(self):
        return f"EngineTest(codex={self.gene_type})"
