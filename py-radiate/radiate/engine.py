from typing import Any, Callable, List, Tuple
from .selector import Selector, TournamentSelector, RouletteSelector
from .alterer import Alterer, UniformCrossover, UniformMutator
from .diversity import Diversity, Hammingdistance, EuclideanDistance
from ._typing import GeneType, ObjectiveType
from .codec import FloatCodec, IntCodec, CharCodec, BitCodec
from .limit import Limit

from radiate.radiate import (
    PyEngineBuilder,
    PyGeneration,
    PyEngine,
)


class GeneticEngine:
    """
    Genetic Engine for optimization problems.
    This class serves as the main interface for running genetic algorithms, allowing
    the customization of various parameters of the engine.
    """

    def __init__(
        self,
        codec: FloatCodec | IntCodec | CharCodec | BitCodec,
        fitness_func: Callable[[Any], Any],
        offspring_selector: Selector | None = None,
        survivor_selector: Selector | None = None,
        alters: None | Alterer | List[Alterer] = None,
        diversity: None | Hammingdistance | EuclideanDistance = None,
        population_size: int = 100,
        offspring_fraction: float = 0.8,
        max_phenotype_age: int = 20,
        max_species_age: int = 20,
        species_threshold: float = 1.5,
        objectives: str | List[str] = ObjectiveType.MIN,
        num_threads: int = 1,
        front_range: Tuple[int, int] | None = (800, 900),
    ):
        self.codec = codec
        self.fitness_func = fitness_func

        if isinstance(self.codec, FloatCodec):
            self.gene_type = GeneType.FLOAT()
        elif isinstance(self.codec, IntCodec):
            self.gene_type = GeneType.INT()
        elif isinstance(self.codec, CharCodec):
            self.gene_type = GeneType.CHAR()
        elif isinstance(self.codec, BitCodec):
            self.gene_type = GeneType.BIT()
        else:
            raise TypeError(f"Codec type {type(self.codec)} is not supported.")

        survivor_selector = self.__get_params(
            survivor_selector or TournamentSelector(k=3)
        )
        offspring_selector = self.__get_params(offspring_selector or RouletteSelector())
        alters = self.__get_params(alters or [UniformCrossover(), UniformMutator()])
        diversity = self.__get_params(diversity, allow_none=True)
        objectives = self.__get_objectives(objectives)
        front_range = self.__get_front_range(front_range)

        self.builder = PyEngineBuilder(
            objectives=objectives,
            survivor_selector=survivor_selector,
            offspring_selector=offspring_selector,
            alters=alters,
            population_size=population_size,
            offspring_fraction=offspring_fraction,
            num_threads=num_threads,
            front_range=front_range,
            diversity=diversity,
            max_phenotype_age=max_phenotype_age,
            max_species_age=max_species_age,
            species_threshold=species_threshold,
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

    def diversity(self, diversity: Diversity, species_threshold: float = 1.5):
        """Set the diversity."""
        if diversity is None:
            raise ValueError("Diversity must be provided.")
        self.builder.set_diversity(self.__get_params(diversity, allow_none=True))
        self.builder.set_species_threshold(species_threshold)

    def offspring_fraction(self, fraction: float):
        """Set the offspring fraction."""
        if not (0 < fraction <= 1):
            raise ValueError("Offspring fraction must be between 0 and 1.")
        self.builder.set_offspring_fraction(fraction)

    def max_age(self, max_phenotype_age: int = 20, max_species_age: int = 20):
        """Set the maximum age for phenotypes and species."""
        if max_phenotype_age <= 0 or max_species_age <= 0:
            raise ValueError("Maximum age must be greater than 0.")
        self.builder.set_max_phenotype_age(max_phenotype_age)
        self.builder.set_max_species_age(max_species_age)

    def minimizing(self):
        """Set the objectives."""
        self.builder.set_objectives(self.__get_objectives(ObjectiveType.MIN))

    def maximizing(self):
        """Set the objectives."""
        self.builder.set_objectives(self.__get_objectives(ObjectiveType.MAX))

    def multi_objective(
        self, objectives: List[str], front_range: Tuple[int, int] | None = None
    ):
        """Set the objectives for a multiobjective problem"""
        if not isinstance(objectives, list) or not all(
            obj in [ObjectiveType.MIN, ObjectiveType.MAX] for obj in objectives
        ):
            raise ValueError("Objectives must be a list of 'min' or 'max'.")
        self.builder.set_objectives(self.__get_objectives(objectives))
        self.builder.set_front_range(self.__get_front_range(front_range))

    def num_threads(self, num_threads: int):
        """Set the number of threads."""
        if num_threads <= 0:
            raise ValueError("Number of threads must be greater than 0.")
        self.builder.set_num_threads(num_threads)

    def __get_engine(self):
        """Get the engine."""
        return PyEngine(
            self.gene_type.gene_type, self.codec.codec, self.fitness_func, self.builder
        )

    def __get_front_range(self, front_range: Tuple[int, int] | None) -> Tuple[int, int]:
        """Get the front range."""
        if front_range is None:
            return (800, 900)
        if not isinstance(front_range, tuple) or len(front_range) != 2:
            raise ValueError("Front range must be a tuple of (min, max).")
        if front_range[0] >= front_range[1]:
            raise ValueError(
                "Minimum front range must be less than maximum front range."
            )
        return front_range

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

    def __get_params(
        self,
        value: Selector | Diversity | Alterer | List[Alterer],
        allow_none: bool = False,
    ) -> List[Any] | None:
        """Get the parameters from the value."""
        if isinstance(value, Selector):
            return value.params
        if isinstance(value, Alterer):
            return [value.params]
        if isinstance(value, Diversity):
            if not value.is_valid(self.gene_type):
                raise TypeError(
                    f"Diversity {value} is not valid for genome type {self.gene_type.gene_type}."
                )
            return value.params
        if isinstance(value, list):
            if all(isinstance(alter, Alterer) for alter in value):
                for alter in value:
                    if not alter.is_valid(self.gene_type):
                        raise TypeError(
                            f"Alterer {alter} is not valid for genome type {self.gene_type.gene_type}."
                        )
                return [alter.params for alter in value]
            
        if allow_none and value is None:
            return None
        else:
            raise TypeError(f"Param type {type(value)} is not supported.")

    def __repr__(self):
        return f"EngineTest(codec={self.gene_type})"
