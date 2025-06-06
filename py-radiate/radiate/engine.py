from typing import Any, Callable, List, Tuple
from .selector import SelectorBase, TournamentSelector, RouletteSelector
from .alterer import AlterBase, UniformCrossover, UniformMutator
from .diversity import DiversityBase
from .codec import CodecBase
from .limit import LimitBase
from .generation import Generation


from .radiate import PyEngineBuilder, PyObjective, PySubscriber


class GeneticEngine:
    """
    Genetic Engine for optimization problems.
    This class serves as the main interface for running genetic algorithms, allowing
    the customization of various parameters of the engine.
    """

    def __init__(
        self,
        codec: CodecBase,
        fitness_func: Callable[[Any], Any],
        offspring_selector: SelectorBase | None = None,
        survivor_selector: SelectorBase | None = None,
        alters: None | AlterBase | List[AlterBase] = None,
        diversity: None | DiversityBase = None,
        population_size: int = 100,
        offspring_fraction: float = 0.8,
        max_phenotype_age: int = 20,
        max_species_age: int = 20,
        species_threshold: float = 1.5,
        objectives: str | List[str] = ["min"],
        num_threads: int = 1,
        front_range: Tuple[int, int] | None = (800, 900),
        subscribe: List[Callable[[Any], None]] | Callable[[Any], None] | None = None,
    ):
        self.engine = None

        survivor_selector = self.__get_params(
            survivor_selector or TournamentSelector(k=3)
        )
        offspring_selector = self.__get_params(offspring_selector or RouletteSelector())
        alters = self.__get_params(alters or [UniformCrossover(), UniformMutator()])
        diversity = self.__get_params(diversity, allow_none=True)
        objectives = self.__get_objectives(objectives)
        front_range = self.__get_front_range(front_range)

        codec = self.__get_codec(codec)
        fitness_func = fitness_func

        self.builder = PyEngineBuilder(
            fitness_func,
            codec,
            population_size=population_size,
            offspring_fraction=offspring_fraction,
            objective=objectives,
            front_range=front_range,
            num_threads=num_threads,
            max_phenotype_age=max_phenotype_age,
            max_species_age=max_species_age,
            species_threshold=species_threshold,
            alters=alters,
            offspring_selector=offspring_selector,
            survivor_selector=survivor_selector,
            diversity=diversity,
        )

    def __repr__(self):
        if self.engine is None:
            return f"{self.builder.__repr__()}"
        return f"{self.engine.__repr__()}"

    def __dict__(self):
        """Return the internal state of the engine builder for debugging."""
        return self.builder.__dict__()

    def run(
        self, limits: LimitBase | List[LimitBase] | None = None, log: bool = False
    ) -> Generation:
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
            self.limits(limits)
        else:
            limits = self.builder.get_limits()
            if not limits or len(limits) == 0:
                raise ValueError(
                    "At least one limit must be provided to run the engine."
                )

        self.engine = self.builder.build()
        return Generation(self.engine.run(log=log))

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
        self.builder.set_survivor_selector(self.__get_params(selector))

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
        self.builder.set_offspring_selector(self.__get_params(selector))

    def alters(self, alters: AlterBase | List[AlterBase]):
        """Set the alters.
        Args:
            alters (AlterBase | List[AlterBase]): The alterers to use in the engine.
        Raises:
            ValueError: If alters is None or invalid.

        Example:
        ---------
        >>> engine.alters([rd.SimulatedBinaryCrossover(1.0, 1.0), rd.UniformMutator(0.1)])
        """
        if alters is None:
            raise ValueError("Alters must be provided.")
        self.builder.set_alters(self.__get_params(alters))

    def limits(self, limits: LimitBase | List[LimitBase]):
        """Set the limits.
        Args:
            limits (LimitBase | List[LimitBase]): The limits to apply to the engine.
        Raises:
            ValueError: If limits is None or invalid.

        Example:
        ---------
        >>> engine.limits(rd.ScoreLimit(0.0001))
        """
        if limits is None:
            raise ValueError("Limits must be provided.")
        lims = [lim.limit for lim in (limits if isinstance(limits, list) else [limits])]
        self.builder.set_limits(lims)

    def diversity(self, diversity: DiversityBase, species_threshold: float = 1.5):
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
        self.builder.set_diversity(self.__get_params(diversity, allow_none=True))
        self.builder.set_species_threshold(species_threshold)

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
        self.builder.set_max_phenotype_age(max_phenotype_age)
        self.builder.set_max_species_age(max_species_age)

    def minimizing(self):
        """Set the objectives to minimize.

        Example:
        ---------
        >>> engine.minimizing()
        """
        self.builder.set_objective(PyObjective.min())

    def maximizing(self):
        """Set the objectives to maximize.

        Example:
        ---------
        >>> engine.maximizing()
        """
        self.builder.set_objective(PyObjective.max())

    def multi_objective(
        self, objectives: List[str], front_range: Tuple[int, int] | None = None
    ):
        """Set the objectives for a multiobjective problem.
        Args:
            objectives (List[str]): A list of objectives, each being 'min' or 'max'.
            front_range (Tuple[int, int] | None): The range for the Pareto front.
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

        self.builder.set_objective(self.__get_objectives(objectives))
        self.builder.set_front_range(self.__get_front_range(front_range))

    def num_threads(self, num_threads: int):
        """Set the number of threads.
        Args:
            num_threads (int): The number of threads to use.
        Raises:
            ValueError: If num_threads is less than or equal to 0.
        Example:
        ---------
        >>> engine.num_threads(4)
        """
        if num_threads <= 0:
            raise ValueError("Number of threads must be greater than 0.")
        self.builder.set_num_threads(num_threads)

    def subscribe(
        self, event_handler: List[Callable[[Any], None]] | Callable[[Any], None]
    ):
        """Register an event handler.
        Args:
            event_handler (Callable[[Any], None] | List[Callable[[Any], None]]): The event handler(s) to register.
        Raises:
            TypeError: If event_handler is not callable or a list of callables.

        Example:
        ---------
        >>> engine.subscribe(my_event_handler)
        >>> engine.subscribe([handler1, handler2])
        """
        if callable(event_handler):
            self.builder.set_subscribers([PySubscriber(event_handler)])
        elif all(callable(handler) for handler in event_handler):
            self.builder.set_subscribers(
                [PySubscriber(handler) for handler in event_handler]
            )
        else:
            raise TypeError("Event handler must be a callable or a list of callables.")

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

    def __get_objectives(self, objectives: str | List[str]) -> PyObjective:
        """Get the objectives."""
        if objectives is None:
            raise ValueError("Objectives must be provided.")
        if isinstance(objectives, str):
            if objectives not in ["min", "max"]:
                raise ValueError("Objectives must be 'min' or 'max'.")
            return PyObjective([objectives])
        if isinstance(objectives, list):
            for obj in objectives:
                if obj not in ["min", "max"]:
                    raise ValueError("Objectives must be 'min' or 'max'.")
            return PyObjective.multi(objectives)
        raise TypeError(f"Objectives type {type(objectives)} is not supported.")

    def __get_params(
        self,
        value: SelectorBase | DiversityBase | AlterBase | List[AlterBase],
        allow_none: bool = False,
    ) -> List[Any] | None:
        """Get the parameters from the value."""
        if isinstance(value, SelectorBase):
            return value.selector
        if isinstance(value, AlterBase):
            return [value.alterer]
        if isinstance(value, DiversityBase):
            return value.diversity
        if isinstance(value, list):
            if all(isinstance(alter, AlterBase) for alter in value):
                return [alter.alterer for alter in value]

        if allow_none and value is None:
            return None
        else:
            raise TypeError(f"Param type {type(value)} is not supported.")

    def __get_codec(self, codec: CodecBase | Callable[[], List[Any]]) -> Any:
        """Get the codec."""
        from .codec import FloatCodec, IntCodec, CharCodec, BitCodec

        if isinstance(codec, FloatCodec):
            return codec.codec
        if isinstance(codec, IntCodec):
            return codec.codec
        if isinstance(codec, CharCodec):
            return codec.codec
        if isinstance(codec, BitCodec):
            return codec.codec

        else:
            raise TypeError(
                f"Codec type {type(codec)} is not supported. "
                "Use FloatCodec, IntCodec, CharCodec, or BitCodec."
            )
