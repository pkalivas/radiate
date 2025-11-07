from typing import Any, Callable

from ._typing import Subscriber
from .builder import EngineBuilder
from .generation import Generation
from .codec import (
    FloatCodec,
    IntCodec,
    CharCodec,
    BitCodec,
    GraphCodec,
    CodecBase,
    PermutationCodec,
    TreeCodec,
    AnyCodec,
)

from .inputs.input import EngineInput, EngineInputType
from .inputs.selector import SelectorBase, TournamentSelector, RouletteSelector
from .inputs.alterer import AlterBase
from .inputs.distance import DistanceBase
from .inputs.executor import Executor
from .fitness import FitnessBase
from .inputs.limit import LimitBase

from .genome import GeneType
from .genome.population import Population


class GeneticEngine[G, T]:
    """
    Genetic Engine for optimization problems.
    This class serves as the main interface for running genetic algorithms, allowing
    the customization of various parameters of the engine.
    """

    def __init__(
        self,
        codec: CodecBase[G, T],
        fitness_func: Callable[[T], Any] | FitnessBase[T],
        population: Population[G] | None = None,
        offspring_selector: SelectorBase | None = None,
        survivor_selector: SelectorBase | None = None,
        alters: AlterBase | list[AlterBase] | None = None,
        diversity: DistanceBase | None = None,
        population_size: int = 100,
        offspring_fraction: float = 0.8,
        max_phenotype_age: int = 20,
        max_species_age: int = 20,
        species_threshold: float = 0.5,
        objective: str | list[str] = "max",
        executor: Executor | None = None,
        front_range: tuple[int, int] | None = (800, 900),
        subscribe: Subscriber | None = None,
    ):
        self.gene_type = None
        if isinstance(codec, IntCodec):
            self.gene_type = GeneType.INT
        elif isinstance(codec, FloatCodec):
            self.gene_type = GeneType.FLOAT
        elif isinstance(codec, CharCodec):
            self.gene_type = GeneType.CHAR
        elif isinstance(codec, BitCodec):
            self.gene_type = GeneType.BIT
        elif isinstance(codec, GraphCodec):
            self.gene_type = GeneType.GRAPH
        elif isinstance(codec, TreeCodec):
            self.gene_type = GeneType.TREE
        elif isinstance(codec, PermutationCodec):
            self.gene_type = GeneType.PERMUTATION
        elif isinstance(codec, AnyCodec):
            self.gene_type = GeneType.ANY
        else:
            raise TypeError(f"Codec type {type(codec)} is not supported.")

        if fitness_func is None:
            raise ValueError("Fitness function must be provided.")

        self.builder = EngineBuilder(self.gene_type, codec, fitness_func)

        self.builder.set_population(population)
        self.builder.set_survivor_selector(survivor_selector or TournamentSelector(k=3))
        self.builder.set_offspring_selector(offspring_selector or RouletteSelector())
        self.builder.set_alters(alters)
        self.builder.set_diversity(diversity, species_threshold)
        self.builder.set_population_size(population_size)
        self.builder.set_offspring_fraction(offspring_fraction)
        self.builder.set_max_age(max_phenotype_age)
        self.builder.set_max_species_age(max_species_age)
        self.builder.set_objective(objective, front_range)
        self.builder.set_executor(executor)
        self.builder.set_subscribers(subscribe)

    def __repr__(self):
        if self.engine is None:
            return f"{self.builder.__repr__()}"
        return f"{self.engine.__repr__()}"

    def __dict__(self):
        """Return the internal state of the engine builder for debugging."""
        return self.builder.__dict__

    def __iter__(self):
        """Return an iterator over the engine's generations."""
        self.engine = self.builder.build()
        return self

    def __next__(self):
        """Return the next generation from the engine."""
        if not hasattr(self, "engine"):
            self.engine = self.builder.build()
        return Generation.from_rust(self.engine.next())

    def run(
        self, limits: LimitBase | list[LimitBase], log: bool = False
    ) -> Generation[T]:
        """Run the engine with the given limits.
        Args:
            limits: A single Limit or a list of Limits to apply to the engine.
            log: If True, enables logging for the generation process.
        Returns:
            Generation: The resulting generation after running the engine.
        Raises:
            ValueError: If limits are not provided or invalid.

        Example:
        ---------
        >>> engine.run(rd.ScoreLimit(0.0001), log=True)
        """

        if limits is not None:
            if isinstance(limits, LimitBase):
                limits = [limits]
            elif isinstance(limits, list):
                if len(limits) == 0:
                    raise ValueError(
                        "At least one limit must be provided to run the engine."
                    )
            else:
                raise TypeError(
                    "Limits must be a LimitBase or a list of LimitBase instances."
                )
        else:
            raise ValueError("At least one limit must be provided to run the engine.")

        engine = self.builder.build()

        limit_inputs = [
            EngineInput(
                input_type=EngineInputType.Limit,
                component=lim.component,
                allowed_genes=[self.gene_type],
                **lim.args,
            ).__backend__()
            for lim in limits
        ]

        return Generation.from_rust(engine.run(limit_inputs, log))

    def population_size(self, size: int):
        """Set the population size.
        Args:
            size (int): The size of the population.
        Raises:
            ValueError: If size is less than or equal to 0.

        Example:
        ---------
        >>> engine.population_size(200)
        """
        if size <= 0:
            raise ValueError("Population size must be greater than 0.")
        self.builder.set_population_size(size)

    def survivor_selector(self, selector: SelectorBase):
        """Set the survivor selector.
        Args:
            selector (SelectorBase): The selector to use for survivors.
        Raises:
            ValueError: If selector is None or invalid.

        Example:
        ---------
        >>> engine.survivor_selector(rd.TournamentSelector(k=5))
        """
        if selector is None:
            raise ValueError("Selector must be provided.")
        self.builder.set_survivor_selector(selector)

    def offspring_selector(self, selector: SelectorBase):
        """Set the offspring selector.
        Args:
            selector (SelectorBase): The selector to use for offspring.
        Raises:
            ValueError: If selector is None or invalid.

        Example:
        ---------
        >>> engine.offspring_selector(rd.RouletteSelector())
        """
        if selector is None:
            raise ValueError("Selector must be provided.")
        self.builder.set_offspring_selector(selector)

    def alters(self, alters: AlterBase | list[AlterBase]):
        """Set the alters.
        Args:
            alters (AlterBase | list[AlterBase]): The alterers to use in the engine.
        Raises:
            ValueError: If alters is None or invalid.

        Example:
        ---------
        >>> engine.alters([rd.SimulatedBinaryCrossover(1.0, 1.0), rd.UniformMutator(0.1)])
        """
        if alters is None:
            raise ValueError("Alters must be provided.")
        self.builder.set_alters(alters)

    def diversity(self, diversity: DistanceBase, species_threshold: float = 1.5):
        """Set the diversity.
        Args:
            diversity (DiversityBase): The diversity strategy to use.
            species_threshold (float): The threshold for species diversity.
        Raises:
            ValueError: If diversity is None or invalid.

        Example:
        ---------
        >>> engine.diversity(rd.SpeciesDiversity(), species_threshold=1.5)
        """
        if diversity is None:
            raise ValueError("Diversity must be provided.")
        if species_threshold <= 0:
            raise ValueError("Species threshold must be greater than 0.")
        self.builder.set_diversity(diversity, species_threshold)

    def offspring_fraction(self, fraction: float):
        """Set the offspring fraction.
        Args:
            fraction (float): The fraction of offspring to create.
        Raises:
            ValueError: If fraction is not between 0 and 1.

        Example:
        ---------
        >>> engine.offspring_fraction(0.8)
        """
        if not (0 < fraction <= 1):
            raise ValueError("Offspring fraction must be between 0 and 1.")
        self.builder.set_offspring_fraction(fraction)

    def max_age(self, max_phenotype_age: int = 20, max_species_age: int = 20):
        """Set the maximum age for phenotypes and species.
        Args:
            max_phenotype_age (int): The maximum age for phenotypes.
            max_species_age (int): The maximum age for species.
        Raises:
            ValueError: If max_phenotype_age or max_species_age is less than or equal to 0.

        Example:
        ---------
        >>> engine.max_age(max_phenotype_age=30, max_species_age=25)
        """
        if max_phenotype_age <= 0 or max_species_age <= 0:
            raise ValueError("Maximum age must be greater than 0.")
        self.builder.set_max_age(max_phenotype_age)
        self.builder.set_max_species_age(max_species_age)

    def minimizing(self):
        """Set the objectives to minimize.

        Example:
        ---------
        >>> engine.minimizing()
        """
        self.builder.set_objective(["min"], None)

    def maximizing(self):
        """Set the objectives to maximize.

        Example:
        ---------
        >>> engine.maximizing()
        """
        self.builder.set_objective(["max"], None)

    def multi_objective(
        self, objectives: list[str], front_range: tuple[int, int] | None = None
    ):
        """Set the objectives for a multiobjective problem.
        Args:
            objectives (list[str]): A list of objectives, each being 'min' or 'max'.
            front_range (tuple[int, int] | None): The range for the Pareto front.
        Raises:
            ValueError: If objectives is not a list of 'min' or 'max', or if front_range is invalid.

        Example:
        ---------
        >>> engine.multi_objective(["min", "max"], front_range=(800, 900))
        """
        if not isinstance(objectives, list) or not all(
            obj in ["min", "max"] for obj in objectives
        ):
            raise ValueError("Objectives must be a list of 'min' or 'max'.")
        self.builder.set_objective(objectives, front_range)

    def executor(self, executor: Executor):
        """Set the executor.
        Args:
            executor (Executor): The executor to use.
        Example:
        ---------
        >>> engine.executor(Executor.worker_pool())
        """
        if not isinstance(executor, Executor):
            raise TypeError("Executor must be an instance of Executor.")
        self.builder.set_executor(executor)

    def subscribe(self, event_handler: Subscriber | None = None):
        """Register an event handler.
        Args:
            event_handler: Union[
                Callable[[Any], None], list[Callable[[Any], None]], EventHandler, list[EventHandler]
            ] | None: The event handler(s) to register.
        Raises:
            TypeError: If event_handler is not callable or a list of callables.

        Example:
        ---------
        >>> engine.subscribe(my_event_handler)
        >>> engine.subscribe([handler1, handler2])
        """
        self.builder.set_subscribers(event_handler)
