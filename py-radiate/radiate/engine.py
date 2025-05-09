from typing import Any, Callable, List
from .selector import Selector, TournamentSelector, RouletteSelector
from .alterer import Alterer, UniformCrossover, UniformMutator
from ._typing import GeneType
from .codex import FloatCodex, IntCodex 
from .limit import Limit

from radiate.radiate import (
    PyEngineBuilder,
    PyFloatEngine, 
    PyIntEngine
)

class Engine:

    def __init__(self, 
                 codex: FloatCodex | IntCodex,
                 fitness_func: Callable[[Any], Any],
                 offspring_selector: Selector = None,
                 survivor_selector: Selector = None,
                 alters: None | Alterer | List[Alterer] = None,
                 population_size: int = 100):
        self.codex = codex
        self.fitness_func = fitness_func

        if isinstance(self.codex, FloatCodex):
            self.gene_type = GeneType.FLOAT
        elif isinstance(self.codex, IntCodex):
            self.gene_type = GeneType.INT
        else:
            raise TypeError(f"Codex type {type(self.codex)} is not supported.")

        survivor_selector = self.__get_params(survivor_selector or TournamentSelector(k=3))
        offspring_selector = self.__get_params(offspring_selector or RouletteSelector())
        alters = self.__get_params(alters or [UniformCrossover(), UniformMutator()])

        self.builder = PyEngineBuilder(
            objectives=['min'],
            survivor_selector=survivor_selector,
            offspring_selector=offspring_selector,
            alters=alters,
            population_size=population_size,
        )

    def run(self, limits: Limit | List[Limit]):
        limits = [lim.params for lim in (limits if isinstance(limits, list) else [limits])]
        engine = self.__get_engine()
        engine.run(limits)

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

    def __get_engine(self):
        """Get the engine."""
        if self.gene_type == GeneType.FLOAT:
            return PyFloatEngine(self.codex.codex, self.fitness_func, self.builder)
        elif self.gene_type == GeneType.INT:
            return PyIntEngine(self.codex.codex, self.fitness_func, self.builder)
        else:
            raise TypeError(f"Gene type {self.gene_type} is not supported.")

    def __get_params(self, value):
        if isinstance(value, Selector):
            return value.params
        if isinstance(value, Alterer):
            return [value.params]
        if isinstance(value, list):
            if all(isinstance(alter, Alterer) for alter in value):
                for alter in value:
                    if not alter.is_valid(self.gene_type):
                        raise TypeError(f"Alterer {alter} is not valid for genome type {GeneType.FLOAT}.")
                return [alter.params for alter in value]
        raise TypeError(f"Param type {type(value)} is not supported.")
    

    def __repr__(self):
        return f"EngineTest(codex={self.gene_type})"
    

